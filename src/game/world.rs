use std::{
	collections::{VecDeque, HashMap},
	fs::{File, OpenOptions},
	path::Path,
	rc::{Rc, Weak},
	cell::RefCell, time::Duration,
};

use winit::event::WindowEvent;
use nalgebra::Vector3;
use anyhow::Result;

use super::{
	chunk::{Chunk, LoadedChunk, ChunkData},
	entity::Entity,
	block::BlockFace,
	worldgen::WorldGenerator,
	player::{Player, PlayerId}, CHUNK_SIZE,
};
use crate::prelude::*;

// max size of world in chunks
// 16,384 meters in each x and y direction
// 2,048 meters in z direction
pub const WORLD_MAX_SIZE: Vector3<u64> = Vector3::new(512, 64, 512);

pub struct World {
	self_weak: Weak<Self>,
	players: RefCell<HashMap<PlayerId, Player>>,
	entities: RefCell<Vec<Box<dyn Entity>>>,
	pub chunks: RefCell<HashMap<ChunkPos, RefCell<LoadedChunk>>>,
	cached_chunks: RefCell<HashMap<ChunkPos, ChunkData>>,
	world_generator: WorldGenerator,
	// backing file of the world
	file: File,
}

impl World {
	pub fn load_from_file<T: AsRef<Path>>(file_name: T) -> Result<Rc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(file_name)?;

		Ok(Rc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RefCell::new(HashMap::new()),
			entities: RefCell::new(Vec::new()),
			chunks: RefCell::new(HashMap::new()),
			cached_chunks: RefCell::new(HashMap::new()),
			world_generator: WorldGenerator::new(),
			file,
		}))
	}

	// TEMP
	pub fn new_test() -> Result<Rc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open("test-world")?;

		let out = Rc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RefCell::new(HashMap::new()),
			entities: RefCell::new(Vec::new()),
			chunks: RefCell::new(HashMap::new()),
			cached_chunks: RefCell::new(HashMap::new()),
			world_generator: WorldGenerator::new(),
			file,
		});

		Ok(out)
	}

	// TODO: load and unload queue mesh updates
	// loads all chunks between min_chunk and max_chunk not including max_chunk,
	// or incraments the load count if they are already loaded
	pub fn load_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		let mut chunks = self.chunks.borrow_mut();

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					let chunk = chunks.entry(position)
						.or_insert_with(|| self.world_generator
							.generate_chunk(self.self_weak.upgrade().unwrap(), position));

					// when first inserting load count starts at 0
					chunk.borrow_mut().load_count += 1;
				}
			}
		}
	}

	// decraments the load counter of all chunks between min and max chunk, not including max
	// and unloads them if the count reaches 0
	pub fn unload_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		let mut chunks = self.chunks.borrow_mut();

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					if let Some(loaded_chunk) = chunks.get(&position) {
						let mut loaded_chunk = loaded_chunk.borrow_mut();
						loaded_chunk.load_count -= 1;
						if loaded_chunk.load_count == 0 {
							drop(loaded_chunk);
							chunks.remove(&position);
						}
					}
				}
			}
		}
	}

	// performs a block mesh update on all blocke between min_block inclusive and max_block exclusive
	pub fn mesh_update(&self, min_block: BlockPos, max_block: BlockPos) {
		let chunks = self.chunks.borrow();
		for x in min_block.x..max_block.x {
			for y in min_block.y..max_block.y {
				for z in min_block.z..max_block.z {
					let block = BlockPos::new(x, y, z);
					if let Some(chunk) = chunks.get(&block.into_chunk_pos()) {
						chunk.borrow_mut().chunk.mesh_update(block.make_chunk_local());
					}
				}
			}
		}
	}

	// FIXME: this is ugly
	pub fn chunk_mesh_update(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		let chunks = self.chunks.borrow();

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let chunk_pos = ChunkPos::new(x, y, z);

					if let Some(chunk) = chunks.get(&chunk_pos) {
						let mut chunk = chunk.borrow_mut();
						for x in 0..CHUNK_SIZE as i64 {
							for y in 0..CHUNK_SIZE as i64 {
								for z in 0..CHUNK_SIZE as i64 {
									chunk.chunk.mesh_update(BlockPos::new(x, y, z));
								}
							}
						}
					}
				}
			}
		}
	}
}

// NOTE: when multiplayer is implemented, all of the methods in the impl block below 
// will be all the different type of requests that could be sent by the client to the server
impl World {
	pub fn connect(&self) -> PlayerId {
		let player = Player::new();

		let min_load_chunk = player.chunk_position() - player.render_distance();
		let max_load_chunk = player.chunk_position() + player.render_distance();
		self.load_chunks(min_load_chunk, max_load_chunk);
		self.chunk_mesh_update(min_load_chunk, max_load_chunk);

		let id = player.id();
		self.players.borrow_mut().insert(id, player);
		id
	}

	// TODO: allow changing from more than 1 chunk at at a time
	// TODO: when going along diaganols, sometimes chunks are loaded and immediately unloaded
	// TEMP: returns true if mesh has changed
	pub fn set_player_position(&self, player_id: PlayerId, position: Position) -> Option<bool> {
		let mut players = self.players.borrow_mut();
		let player = players.get_mut(&player_id)?;

		let chunk_position = position.into_chunk_pos();

		let render_zone_corner = player.chunk_position() - player.render_distance();
		let render_zone_length = 2 * player.render_distance();

		if chunk_position.x != player.chunk_position().x {
			let xaxis = ChunkPos::new(1, 0, 0);

			let pos_min_chunk = render_zone_corner + render_zone_length.xonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + xaxis;

			let neg_min_chunk = render_zone_corner - xaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.yzonly();

			if chunk_position.x == player.chunk_position().x + 1 {
				let neg_min_chunk = neg_min_chunk + xaxis;
				let neg_max_chunk = neg_max_chunk + xaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.x == player.chunk_position().x - 1 {
				let pos_min_chunk = pos_min_chunk - xaxis;
				let pos_max_chunk = pos_max_chunk - xaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		if chunk_position.y != player.chunk_position().y {
			let yaxis = ChunkPos::new(0, 1, 0);

			let pos_min_chunk = render_zone_corner + render_zone_length.yonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + yaxis;

			let neg_min_chunk = render_zone_corner - yaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.xzonly();

			if chunk_position.y == player.chunk_position().y + 1 {
				let neg_min_chunk = neg_min_chunk + yaxis;
				let neg_max_chunk = neg_max_chunk + yaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.y == player.chunk_position().y - 1 {
				let pos_min_chunk = pos_min_chunk - yaxis;
				let pos_max_chunk = pos_max_chunk - yaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		if chunk_position.z != player.chunk_position().z {
			let zaxis = ChunkPos::new(0, 0, 1);

			let pos_min_chunk = render_zone_corner + render_zone_length.zonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + zaxis;

			let neg_min_chunk = render_zone_corner - zaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.xyonly();

			if chunk_position.z == player.chunk_position().z + 1 {
				let neg_min_chunk = neg_min_chunk + zaxis;
				let neg_max_chunk = neg_max_chunk + zaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.z == player.chunk_position().z - 1 {
				let pos_min_chunk = pos_min_chunk - zaxis;
				let pos_max_chunk = pos_max_chunk - zaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		let out = chunk_position != player.chunk_position();

		player.position = position;
		Some(out)
	}

	pub fn world_mesh(&self) -> Vec<BlockFace> {
		let out = self.chunks.borrow().iter()
			.map(|(_, c)| c.borrow().chunk.get_block_mesh())
			.flatten()
			.collect::<Vec<_>>();
		out
	}
}

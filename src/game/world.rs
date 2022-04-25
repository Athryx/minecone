use std::{
	fs::{File, OpenOptions},
	path::Path,
	rc::{Rc, Weak},
	cell::RefCell, time::Duration,
};

use rustc_hash::FxHashMap;
use winit::event::WindowEvent;
use nalgebra::Vector3;
use anyhow::Result;

use super::{
	chunk::{Chunk, LoadedChunk, ChunkData, VisitedBlockMap},
	entity::Entity,
	block::{BlockFaceMesh, BlockFace, Block, Stone},
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
	players: RefCell<FxHashMap<PlayerId, Player>>,
	entities: RefCell<Vec<Box<dyn Entity>>>,
	pub chunks: RefCell<FxHashMap<ChunkPos, RefCell<LoadedChunk>>>,
	cached_chunks: RefCell<FxHashMap<ChunkPos, ChunkData>>,
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
			players: RefCell::new(FxHashMap::default()),
			entities: RefCell::new(Vec::new()),
			chunks: RefCell::new(FxHashMap::default()),
			cached_chunks: RefCell::new(FxHashMap::default()),
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
			players: RefCell::new(FxHashMap::default()),
			entities: RefCell::new(Vec::new()),
			chunks: RefCell::new(FxHashMap::default()),
			cached_chunks: RefCell::new(FxHashMap::default()),
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
		/*let chunks = self.chunks.borrow();
		for x in min_block.x..max_block.x {
			for y in min_block.y..max_block.y {
				for z in min_block.z..max_block.z {
					let block = BlockPos::new(x, y, z);
					if let Some(chunk) = chunks.get(&block.into_chunk_pos()) {
						chunk.borrow_mut().chunk.mesh_update(block.make_chunk_local());
					}
				}
			}
		}*/
	}

	// performs mesh updates on the passed in block as well as all adjacent blocks
	pub fn mesh_update_adjacent(&self, block: BlockPos) {
		let chunks = self.chunks.borrow();

		let block_chunk_local = block.as_chunk_local();
		let mut visit_map = VisitedBlockMap::new();

		if let Some(chunk) = chunks.get(&block.as_chunk_pos()) {
			let mut chunk = chunk.borrow_mut();
			chunk.chunk.mesh_update_inner(BlockFace::XPos, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::XNeg, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YPos, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YNeg, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZPos, block_chunk_local.z as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZNeg, block_chunk_local.z as usize, &mut visit_map);
		}

		for face in BlockFace::iter() {
			// subtract to get opposite as normal offest
			let offset_block = block - face.block_pos_offset();
			if let Some(chunk) = chunks.get(&offset_block.as_chunk_pos()) {
				chunk.borrow_mut().chunk.mesh_update_inner(face,
					offset_block.as_chunk_local().get_face_component(face) as usize, &mut visit_map);
			}
		}
	}

	pub fn chunk_mesh_update(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		let chunks = self.chunks.borrow();

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let chunk_pos = ChunkPos::new(x, y, z);

					if let Some(chunk) = chunks.get(&chunk_pos) {
						chunk.borrow_mut().chunk.chunk_mesh_update();
					}
				}
			}
		}
	}

	pub fn chunk_mesh_update_face(&self, face: BlockFace, min_chunk: ChunkPos, max_chunk: ChunkPos) {
	}

	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&dyn Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(self.chunks.borrow().get(&chunk_position)?
			.borrow().chunk.get_block(block.as_chunk_local())))
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(self.chunks.borrow().get(&chunk_position)?
			.borrow_mut().chunk.get_block_mut(block.as_chunk_local())))
	}

	// sets the block at BlockPos, returns bool on success
	pub fn set_block(&self, block_pos: BlockPos, block: Box<dyn Block>) -> bool {
		let (chunk_pos, block_pos) = block_pos.as_chunk_block_pos();

		let chunks = self.chunks.borrow();
		if let Some(chunk) = chunks.get(&chunk_pos) {
			chunk.borrow_mut().chunk.set_block(block_pos, block);
			true
		} else {
			false
		}
	}

	// casts a ray starting at ray_start up to a length of max_length
	// if a block other than air is found, the coordinates are returned, otherwise None is returned
	// if the ray ever intersects with an empty chunk, None is returned
	pub fn block_raycast(&self, ray_start: Position, ray: Vector3<f64>, max_length: f64) -> Option<BlockPos> {
		let ray = ray.normalize();
		let block_start_pos = ray_start.into_block_pos();
		let mut block_pos = block_start_pos;

		let direction_x = if ray.x > 0.0 { 1 } else if ray.x < 0.0 { -1 } else { 0 };
		let direction_y = if ray.y > 0.0 { 1 } else if ray.y < 0.0 { -1 } else { 0 };
		let direction_z = if ray.z > 0.0 { 1 } else if ray.z < 0.0 { -1 } else { 0 };

		let intercept_time_interval_x = if ray.x != 0.0 { (1.0 / ray.x).abs() } else { f64::INFINITY };
		let intercept_time_interval_y = if ray.y != 0.0 { (1.0 / ray.y).abs() } else { f64::INFINITY };
		let intercept_time_interval_z = if ray.z != 0.0 { (1.0 / ray.z).abs() } else { f64::INFINITY };

		let ray_offset_x = if ray_start.x > 0.0 { ray_start.x % 1.0 } else { 1.0 + (ray_start.x % 1.0) };
		let ray_offset_y = if ray_start.y > 0.0 { ray_start.y % 1.0 } else { 1.0 + (ray_start.y % 1.0) };
		let ray_offset_z = if ray_start.z > 0.0 { ray_start.z % 1.0 } else { 1.0 + (ray_start.z % 1.0) };

		let mut next_intercept_time_x = if ray.x > 0.0 { (1.0 - ray_offset_x) / ray.x } else if ray.x < 0.0 { ray_offset_x / -ray.x } else { f64::INFINITY };
		let mut next_intercept_time_y = if ray.y > 0.0 { (1.0 - ray_offset_y) / ray.y } else if ray.y < 0.0 { ray_offset_y / -ray.y } else { f64::INFINITY };
		let mut next_intercept_time_z = if ray.z > 0.0 { (1.0 - ray_offset_z) / ray.z } else if ray.z < 0.0 { ray_offset_z / -ray.z } else { f64::INFINITY };

		loop {
			if next_intercept_time_x < next_intercept_time_y && next_intercept_time_x < next_intercept_time_z {
				block_pos.x += direction_x;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_x += intercept_time_interval_x;
			} else if next_intercept_time_y < next_intercept_time_z {
				block_pos.y += direction_y;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_y += intercept_time_interval_y;
			} else {
				block_pos.z += direction_z;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_z += intercept_time_interval_z;
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

	pub fn world_mesh(&self) -> Vec<BlockFaceMesh> {
		self.chunks.borrow().iter()
			.flat_map(|(_, c)| c.borrow().chunk.get_chunk_mesh())
			.collect::<Vec<_>>()
	}
}

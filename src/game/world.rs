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
	player::{Player, PlayerId},
};
use crate::prelude::*;

// max size of world in chunks
// 16,384 meters in each x and y direction
// 2,048 meters in z direction
const WORLD_MAX_SIZE: Vector3<u64> = Vector3::new(512, 64, 512);

pub struct World {
	self_weak: Weak<RefCell<Self>>,
	players: HashMap<PlayerId, Player>,
	entities: Vec<Box<dyn Entity>>,
	chunks: HashMap<ChunkPos, RefCell<LoadedChunk>>,
	cached_chunks: HashMap<ChunkPos, ChunkData>,
	world_generator: WorldGenerator,
	// backing file of the world
	file: File,
}

impl World {
	pub fn load_from_file<T: AsRef<Path>>(file_name: T) -> Result<Rc<RefCell<Self>>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(file_name)?;

		Ok(Rc::new_cyclic(|weak| RefCell::new(Self {
			self_weak: weak.clone(),
			players: HashMap::new(),
			entities: Vec::new(),
			chunks: HashMap::new(),
			cached_chunks: HashMap::new(),
			world_generator: WorldGenerator::new(),
			file,
		})))
	}

	// TEMP
	pub fn new_test() -> Result<Rc<RefCell<Self>>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open("test-world")?;

		let out = Rc::new_cyclic(|weak| RefCell::new(Self {
			self_weak: weak.clone(),
			players: HashMap::new(),
			entities: Vec::new(),
			chunks: HashMap::new(),
			cached_chunks: HashMap::new(),
			world_generator: WorldGenerator::new(),
			file,
		}));

		Ok(out)
	}

	// gets a chunk if it is loaded, otherwise returns None
	pub fn get_chunk(&self, position: ChunkPos) -> Option<&RefCell<LoadedChunk>> {
		self.chunks.get(&position)
	}

	// TODO: load and unload queue mesh updates
	// loads all chunks between min_chunk and max_chunk not including max_chunk,
	// or incraments the load count if they are already loaded
	pub fn load_chunks(&mut self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					let chunk = self.chunks.entry(position)
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
	pub fn unload_chunks(&mut self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					if let Some(loaded_chunk) = self.chunks.get(&position) {
						let mut loaded_chunk = loaded_chunk.borrow_mut();
						loaded_chunk.load_count -= 1;
						if loaded_chunk.load_count == 0 {
							drop(loaded_chunk);
							self.chunks.remove(&position);
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
	pub fn connect(&mut self) -> PlayerId {
		let player = Player::new();

		let min_load_chunk = player.chunk_position() - player.render_distance();
		let max_load_chunk = player.chunk_position() + player.render_distance();
		self.load_chunks(min_load_chunk, max_load_chunk);

		let id = player.id();
		self.players.insert(id, player);
		id
	}

	pub fn world_mesh(&self) -> Vec<BlockFace> {
		let out = self.chunks.iter()
			.map(|(_, c)| c.borrow().chunk.generate_block_faces())
			.flatten()
			.collect::<Vec<_>>();
		out
	}
}

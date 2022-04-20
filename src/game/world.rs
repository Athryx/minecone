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

		let position = ChunkPos::new(0, 0, 0);
		let chunk = LoadedChunk::new(Chunk::new_test(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(1, 0, 0);
		let chunk = LoadedChunk::new(Chunk::new_test_air(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(0, 1, 0);
		let chunk = LoadedChunk::new(Chunk::new_test_air(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(1, 1, 0);
		let chunk = LoadedChunk::new(Chunk::new_test_air(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(0, 0, 1);
		let chunk = LoadedChunk::new(Chunk::new_test_air(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(1, 0, 1);
		let chunk = LoadedChunk::new(Chunk::new_test(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(0, 1, 1);
		let chunk = LoadedChunk::new(Chunk::new_test_air(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		let position = ChunkPos::new(1, 1, 1);
		let chunk = LoadedChunk::new(Chunk::new_test(out.clone(), position));
		out.borrow_mut().chunks.insert(position, chunk);

		Ok(out)
	}

	// gets a chunk if it is loaded, otherwise returns None
	pub fn get_chunk(&self, position: ChunkPos) -> Option<&RefCell<LoadedChunk>> {
		self.chunks.get(&position)
	}
}

// NOTE: when multiplayer is implemented, all of the methods in the impl block below 
// will be all the different type of requests that could be sent by the client to the server
impl World {
	pub fn connect(&mut self) -> PlayerId {
		let player = Player::new();
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

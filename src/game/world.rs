use std::{
	collections::{VecDeque, HashMap},
	fs::{File, OpenOptions},
	path::Path,
};

use nalgebra::Vector3;
use anyhow::Result;

use super::{
	chunk::{Chunk, ChunkData},
	entity::Entity,
	block::BlockFace,
};

// max size of world in chunks
// 16,384 meters in each x and y direction
// 2,048 meters in z direction
const WORLD_MAX_SIZE: Vector3<u64> = Vector3::new(512, 64, 512);

struct LoadedChunks {
	// Load distance in x, y, and z directions
	load_distance: Vector3<u64>,
	// TODO: in the future maybe make a 3d queue data structure that doesn't have any layers of indirection to be more cache friendly
	chunks: VecDeque<VecDeque<VecDeque<Chunk>>>,
	world_mesh: Vec<BlockFace>,
}

impl LoadedChunks {
	// TEMP
	fn new_test() -> Self {
		let mut chunks = VecDeque::new();
		chunks.push_back(VecDeque::new());
		chunks[0].push_back(VecDeque::new());
		let chunk = Chunk::new_test();
		let faces = chunk.generate_block_faces();
		chunks[0][0].push_back(chunk);

		LoadedChunks {
			load_distance: Vector3::new(1, 1, 1),
			chunks,
			world_mesh: faces,
		}
	}
}

pub struct World {
	// 1 loaded chunks struct per player
	chunks: Vec<LoadedChunks>,
	entities: Vec<Box<dyn Entity>>,
	// the key is the chunk position
	cached_chunks: HashMap<Vector3<u64>, ChunkData>,
	// backing file of the world
	file: File,
}

impl World {
	pub fn load_from_file<T: AsRef<Path>>(file_name: T) -> Result<Self> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(file_name)?;

		Ok(World {
			chunks: Vec::new(),
			entities: Vec::new(),
			cached_chunks: HashMap::new(),
			file,
		})
	}

	// TEMP
	pub fn new_test() -> Result<Self> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open("test-world")?;

		Ok(World {
			chunks: vec![LoadedChunks::new_test()],
			entities: Vec::new(),
			cached_chunks: HashMap::new(),
			file,
		})
	}

	// TODO: once multiplayer support take in player id
	pub fn world_mesh(&self) -> &[BlockFace] {
		&self.chunks[0].world_mesh
	}
}

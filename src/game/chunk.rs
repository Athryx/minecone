use std::rc::Rc;
use std::cell::RefCell;

use nalgebra::Translation3;

use super::block::{Block, BlockFace};
// TEMP
use super::block::{Air, Stone};
use super::entity::Entity;
use super::world::World;
use crate::prelude::*;

use crate::array3d_init;

pub const CHUNK_SIZE: usize = 32;

pub struct Chunk {
	world: Rc<RefCell<World>>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: Position,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	// store them on heap to avoid stack overflow
	blocks: Box<[[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Chunk {
	// TEMP
	pub fn new_test(world: Rc<RefCell<World>>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;

		let mut blocks = Box::new(array3d_init!(Stone::new()));
		blocks[5][16][24] = Air::new();
		blocks[0][0][0] = Air::new();
		blocks[13][19][0] = Air::new();

		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			blocks,
		}
	}

	pub fn new_test_air(world: Rc<RefCell<World>>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;
		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			blocks: Box::new(array3d_init!(Air::new())),
		}
	}

	fn is_block_in_chunk(block: BlockPos) -> bool {
		block.x >= 0
			&& block.x < CHUNK_SIZE as i64
			&& block.y >= 0
			&& block.y < CHUNK_SIZE as i64
			&& block.z >= 0
			&& block.z < CHUNK_SIZE as i64
	}

	fn make_chunk_local(block: BlockPos) -> BlockPos {
		let x = if block.x >= 0 {
			block.x % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + (block.x % CHUNK_SIZE as i64)
		};

		let y = if block.y >= 0 {
			block.y % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + (block.y % CHUNK_SIZE as i64)
		};

		let z = if block.z >= 0 {
			block.z % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + (block.z % CHUNK_SIZE as i64)
		};

		BlockPos::new(x, y, z)
	}

	fn get_chunk_of_block(&self, block: BlockPos) -> ChunkPos {
		let x = if block.x > 0 {
			block.x / CHUNK_SIZE as i64
		} else {
			(block.x - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		let y = if block.y > 0 {
			block.y / CHUNK_SIZE as i64
		} else {
			(block.y - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		let z = if block.z > 0 {
			block.z / CHUNK_SIZE as i64
		} else {
			(block.z - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		ChunkPos::new(x, y, z) + self.chunk_position
	}

	// calls the function on the given block position
	// the block may be from another chunk
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&dyn Block) -> T {
		if Self::is_block_in_chunk(block) {
			let x: usize = block.x.try_into().unwrap();
			let y: usize = block.y.try_into().unwrap();
			let z: usize = block.z.try_into().unwrap();

			Some(f(&*self.blocks[x][y][z]))
		} else {
			let chunk_position = self.get_chunk_of_block(block);
			let chunk_local_position = Self::make_chunk_local(block);
			let x: usize = chunk_local_position.x.try_into().unwrap();
			let y: usize = chunk_local_position.y.try_into().unwrap();
			let z: usize = chunk_local_position.z.try_into().unwrap();

			Some(f(&*self.world.borrow()
				.get_chunk(chunk_position)?.borrow()
				.chunk.blocks[x][y][z]))
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		if Self::is_block_in_chunk(block) {
			let x: usize = block.x.try_into().unwrap();
			let y: usize = block.y.try_into().unwrap();
			let z: usize = block.z.try_into().unwrap();

			Some(f(&mut *self.blocks[x][y][z]))
		} else {
			let chunk_position = self.get_chunk_of_block(block);
			let chunk_local_position = Self::make_chunk_local(block);
			let x: usize = chunk_local_position.x.try_into().unwrap();
			let y: usize = chunk_local_position.y.try_into().unwrap();
			let z: usize = chunk_local_position.z.try_into().unwrap();

			Some(f(&mut *self.world.borrow()
				.get_chunk(chunk_position)?.borrow_mut()
				.chunk.blocks[x][y][z]))
		}
	}

	pub fn generate_block_faces(&self) -> Vec<BlockFace> {
		let mut out = Vec::new();

		for (x, yblocks) in self.blocks.iter().enumerate() {
			for (y, zblocks) in yblocks.iter().enumerate() {
				for (z, block) in zblocks.iter().enumerate() {
					if block.is_air() {
						continue;
					}

					let mut model = block.model().clone();
					model.translate(&Translation3::new(
						self.position.x + x as f64,
						self.position.y + y as f64,
						self.position.z + z as f64
					));

					let x = x as i64;
					let y = y as i64;
					let z = z as i64;

					self.with_block(BlockPos::new(x - 1, y, z), |block| if block.is_air() {
						out.push(model.xneg);
					});
					self.with_block(BlockPos::new(x + 1, y, z), |block| if block.is_air() {
						out.push(model.xpos);
					});

					self.with_block(BlockPos::new(x, y - 1, z), |block| if block.is_air() {
						out.push(model.yneg);
					});
					self.with_block(BlockPos::new(x, y + 1, z), |block| if block.is_air() {
						out.push(model.ypos);
					});

					self.with_block(BlockPos::new(x, y, z - 1), |block| if block.is_air() {
						out.push(model.zneg);
					});
					self.with_block(BlockPos::new(x, y, z + 1), |block| if block.is_air() {
						out.push(model.zpos);
					});
				}
			}
		}

		out
	}
}

pub struct LoadedChunk {
	pub chunk: Chunk,
	chunk_mesh: Vec<BlockFace>,
	pub load_count: u64,
}

impl LoadedChunk {
	pub fn new(chunk: Chunk) -> RefCell<LoadedChunk> {
		let chunk_mesh = chunk.generate_block_faces();
		RefCell::new(LoadedChunk {
			chunk,
			chunk_mesh,
			load_count: 0,
		})
	}
}

// the entire saved state of the chunk, which is all blocks and entities
// TODO: maybe save chunk mesh to load faster
pub struct ChunkData {
	chunk: Chunk,
	entities: Vec<Box<dyn Entity>>,
}

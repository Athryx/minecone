use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

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
	world: Rc<World>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: Position,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	// store them on heap to avoid stack overflow
	blocks: Box<[[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
	chunk_mesh: HashMap<BlockPos, Vec<BlockFace>>,
}

impl Chunk {
	// TEMP
	pub fn new_test(world: Rc<World>, position: ChunkPos) -> Self {
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
			chunk_mesh: HashMap::new(),
		}
	}

	pub fn new_test_air(world: Rc<World>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;
		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			blocks: Box::new(array3d_init!(Air::new())),
			chunk_mesh: HashMap::new(),
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&dyn Block) -> T {
		if block.is_chunk_local() {
			Some(f(self.get_block(block)))
		} else {
			let chunk_position = block.into_chunk_pos() + self.chunk_position;

			Some(f(self.world
				.chunks.borrow().get(&chunk_position)?.borrow()
				.chunk.get_block(block.make_chunk_local())))
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		if block.is_chunk_local() {
			Some(f(self.get_block_mut(block)))
		} else {
			let chunk_position = block.into_chunk_pos() + self.chunk_position;

			Some(f(self.world
				.chunks.borrow().get(&chunk_position)?.borrow_mut()
				.chunk.get_block_mut(block.make_chunk_local())))
		}
	}

	#[inline]
	pub fn get_block(&self, block: BlockPos) -> &dyn Block {
		let x: usize = block.x.try_into().unwrap();
		let y: usize = block.y.try_into().unwrap();
		let z: usize = block.z.try_into().unwrap();
		&*self.blocks[x][y][z]
	}

	#[inline]
	pub fn get_block_mut(&mut self, block: BlockPos) -> &mut dyn Block {
		let x: usize = block.x.try_into().unwrap();
		let y: usize = block.y.try_into().unwrap();
		let z: usize = block.z.try_into().unwrap();
		&mut *self.blocks[x][y][z]
	}

	// performs a mesh update on the given block
	pub fn mesh_update(&mut self, block_pos: BlockPos) {
		assert!(block_pos.is_chunk_local());

		let x = block_pos.x;
		let y = block_pos.y;
		let z = block_pos.z;

		let block = self.get_block(block_pos);

		if block.is_air() {
			self.chunk_mesh.remove(&block_pos);
			return;
		}

		let mut model = block.model().clone();
		// translate only the faces we need to, no the whole model
		let translation = Translation3::new(
			self.position.x + x as f64,
			self.position.y + y as f64,
			self.position.z + z as f64
		);

		let mut out = Vec::new();

		self.with_block(BlockPos::new(x - 1, y, z), |block| if block.is_air() {
			model.xneg.translate(&translation);
			out.push(model.xneg);
		});
		self.with_block(BlockPos::new(x + 1, y, z), |block| if block.is_air() {
			model.xpos.translate(&translation);
			out.push(model.xpos);
		});

		self.with_block(BlockPos::new(x, y - 1, z), |block| if block.is_air() {
			model.yneg.translate(&translation);
			out.push(model.yneg);
		});
		self.with_block(BlockPos::new(x, y + 1, z), |block| if block.is_air() {
			model.ypos.translate(&translation);
			out.push(model.ypos);
		});

		self.with_block(BlockPos::new(x, y, z - 1), |block| if block.is_air() {
			model.zneg.translate(&translation);
			out.push(model.zneg);
		});
		self.with_block(BlockPos::new(x, y, z + 1), |block| if block.is_air() {
			model.zpos.translate(&translation);
			out.push(model.zpos);
		});

		if !out.is_empty() {
			self.chunk_mesh.insert(block_pos, out);
		} else {
			self.chunk_mesh.remove(&block_pos);
		}
	}

	// TEMP
	pub fn get_block_mesh(&self) -> Vec<BlockFace> {
		self.chunk_mesh.iter().map(|(_, v)| v.iter().map(|b| *b)).flatten().collect::<Vec<_>>()
	}
}

pub struct LoadedChunk {
	pub chunk: Chunk,
	pub load_count: u64,
}

impl LoadedChunk {
	pub fn new(chunk: Chunk) -> RefCell<LoadedChunk> {
		//let chunk_mesh = chunk.generate_block_faces();
		RefCell::new(LoadedChunk {
			chunk,
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

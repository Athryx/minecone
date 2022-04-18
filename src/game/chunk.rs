use std::rc::Rc;
use std::cell::RefCell;

use nalgebra::{Vector3, Translation3};

use super::block::{Block, BlockFace};
// TEMP
use super::block::{Air, Stone};
use super::entity::Entity;
use super::world::World;
use crate::prelude::*;

use crate::array3d_init;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
	world: Rc<RefCell<World>>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: Position,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	blocks: [[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
	air_map: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	// TEMP
	pub fn new_test(world: Rc<RefCell<World>>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;
		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			blocks: array3d_init!(Stone::new()),
			air_map: array3d_init!(false),
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
			blocks: array3d_init!(Air::new()),
			air_map: array3d_init!(true),
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

					if x == 0 || self.air_map[x - 1][y][z] {
						out.push(model.xneg);
					}
					if x == CHUNK_SIZE - 1 || self.air_map[x + 1][y][z] {
						out.push(model.xpos);
					}
					if y == 0 || self.air_map[x][y - 1][z] {
						out.push(model.yneg);
					}
					if y == CHUNK_SIZE - 1 || self.air_map[x][y + 1][z] {
						out.push(model.ypos);
					}
					if z == 0 || self.air_map[x][y][z - 1] {
						out.push(model.zneg);
					}
					if z == CHUNK_SIZE - 1 || self.air_map[x][y][z + 1] {
						out.push(model.zpos);
					}
				}
			}
		}

		out
	}
}

// the entire saved state of the chunk, which is all blocks and entities
pub struct ChunkData {
	chunk: Chunk,
	entities: Vec<Box<dyn Entity>>,
}

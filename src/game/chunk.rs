use std::rc::Rc;
use std::cell::RefCell;

use nalgebra::{Vector3, Translation3};

use super::block::{Block, BlockPos, BlockFace};
// TEMP
use super::block::Stone;
use super::entity::Entity;
use super::world::World;

use crate::array3d_init;

const CHUNK_SIZE: usize = 32;

pub type ChunkPos = Vector3<u64>;

pub struct Chunk {
	world: Rc<RefCell<World>>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: BlockPos,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	blocks: [[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
	air_map: [[[bool; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	// TEMP
	pub fn new_test(world: Rc<RefCell<World>>) -> Self {
		Self {
			world,
			position: BlockPos::new(0.0, 0.0, 0.0),
			chunk_position: ChunkPos::new(0, 0, 0),
			blocks: array3d_init!(Stone::new()),
			air_map: array3d_init!(false),
		}
	}

	pub fn generate_block_faces(&self) -> Vec<BlockFace> {
		let mut out = Vec::new();

		for (x, yblocks) in self.blocks.iter().enumerate() {
			for (y, zblocks) in yblocks.iter().enumerate() {
				for (z, block) in zblocks.iter().enumerate() {
					let mut model = block.model().clone();
					model.translate(&Translation3::new(x as f64, y as f64, z as f64));

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

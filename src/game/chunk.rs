use nalgebra::Vector3;

use super::block::{Block, BlockPos, BlockFace};
// TEMP
use super::block::Stone;
use super::entity::Entity;

use crate::array3d_init;

const CHUNK_SIZE: usize = 32;

pub type ChunkPos = Vector3<u64>;

pub struct Chunk {
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: BlockPos,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	blocks: [[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	// TEMP
	pub fn new_test() -> Self {
		Self {
			position: BlockPos::new(0.0, 0.0, 0.0),
			chunk_position: ChunkPos::new(0, 0, 0),
			blocks: array3d_init!(Stone::new()),
		}
	}

	// TODO: implement this, for now it is just a test which renders every block face
	pub fn render_block_faces(&self) -> Vec<BlockFace> {
		let mut out = Vec::new();

		for (x, yblocks) in self.blocks.iter().enumerate() {
			for (y, zblocks) in yblocks.iter().enumerate() {
				for (z, block) in zblocks.iter().enumerate() {
					let model = block.model();
					out.push(model.xpos);
					out.push(model.xneg);
					out.push(model.ypos);
					out.push(model.yneg);
					out.push(model.zpos);
					out.push(model.zneg);
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

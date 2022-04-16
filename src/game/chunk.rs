use nalgebra::Vector3;

use super::block::{Block, BlockPos};
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
}

// the entire saved state of the chunk, which is all blocks and entities
pub struct ChunkData {
	chunk: Chunk,
	entities: Vec<Box<dyn Entity>>,
}

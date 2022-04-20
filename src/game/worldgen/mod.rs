use std::rc::Rc;
use std::cell::RefCell;

use crate::prelude::*;
use super::chunk::{Chunk, LoadedChunk};
use super::world::World;

pub struct WorldGenerator {}

impl WorldGenerator {
	pub fn new() -> Self {
		WorldGenerator {}
	}

	pub fn generate_chunk(&self, world: Rc<RefCell<World>>, position: ChunkPos) -> RefCell<LoadedChunk> {
		if position.y < 0 {
			LoadedChunk::new(Chunk::new_test(world, position))
		} else {
			LoadedChunk::new(Chunk::new_test_air(world, position))
		}
	}
}

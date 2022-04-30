use std::sync::Arc;

use noise::{Seedable, NoiseFn, OpenSimplex};

use crate::prelude::*;
use super::chunk::{Chunk, LoadedChunk};
use super::world::World;
use super::block::*;

trait NoiseFnBlockPos2D : NoiseFn<[f64; 2]> {
	fn get_block_pos2d(&self, block: BlockPos) -> f64 {
		self.get([block.x as f64, block.z as f64])
	}

	// scales the relavant BlockPos coordinated by scale before getting noise from the noise funtion
	fn get_block_pos2d_scaled(&self, block: BlockPos, scale: f64) -> f64 {
		self.get([block.x as f64 * scale, block.z as f64 * scale])
	}
}

impl<T: NoiseFn<[f64; 2]>> NoiseFnBlockPos2D for T {}

trait NoiseFnBlockPos3D : NoiseFn<[f64; 3]> {
	fn get_block_pos3d(&self, block: BlockPos) -> f64 {
		self.get([block.x as f64, block.y as f64, block.z as f64])
	}

	// scales the relavant BlockPos coordinated by scale before getting noise from the noise funtion
	fn get_block_pos3d_scaled(&self, block: BlockPos, scale: f64) -> f64 {
		self.get([block.x as f64 * scale, block.y as f64 * scale, block.z as f64 * scale])
	}
}

impl<T: NoiseFn<[f64; 3]>> NoiseFnBlockPos3D for T {}

pub struct WorldGenerator {
	height_noise: OpenSimplex,
	biome_heigt_noise: OpenSimplex,
}

impl WorldGenerator {
	pub fn new(seed: u32) -> Self {
		WorldGenerator {
			height_noise: OpenSimplex::new().set_seed(seed),
			biome_heigt_noise: OpenSimplex::new().set_seed(seed + 1),
		}
	}

	fn get_height_noise(&self, block: BlockPos) -> i64 {
		let noise = 10.0 * self.height_noise.get_block_pos2d_scaled(block, 0.01);
		(noise * noise) as i64
	}

	pub fn generate_chunk(&self, world: Arc<World>, position: ChunkPos) -> LoadedChunk {
		LoadedChunk::new(Chunk::new(world, position, |block| {
			if block.y > self.get_height_noise(block) {
				Air::new()
			} else {
				Stone::new()
			}
		}))
	}
}

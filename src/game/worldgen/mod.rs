use std::sync::Arc;

use noise::{Seedable, NoiseFn, OpenSimplex};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use nalgebra::Vector2;

use crate::prelude::*;
use biome::{SurfaceBiome, BiomeNoiseData};
use super::chunk::{Chunk, LoadedChunk};
use super::world::World;
use super::block::*;

mod biome;

type Cache2D = FxHashMap<Vector2<i64>, f64>;
type Cache3D = FxHashMap<BlockPos, f64>;

#[derive(Debug, Default)]
struct NoiseCache {
	height_noise: Cache2D,
	biome_height_noise: Cache2D,
	biome_temp_noise: Cache2D,
	biome_precipitation_noise: Cache2D,
}

struct CachedNoise2D {
	noise: OpenSimplex,
	scale: f64,
}

impl CachedNoise2D {
	fn new(seed: u32, scale: f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
		}
	}

	fn get_block_pos(&self, block: BlockPos, cache: &mut Cache2D) -> f64 {
		*cache.entry(block.xz()).or_insert_with(||
			self.noise.get([block.x as f64 * self.scale, block.z as f64 * self.scale]))
	}
}

struct CachedNoise3D {
	noise: OpenSimplex,
	scale: f64,
}

impl CachedNoise3D {
	fn new(seed: u32, scale: f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
		}
	}

	fn get_block_pos(&self, block: BlockPos, cache: &mut Cache3D) -> f64 {
		*cache.entry(block).or_insert_with(||
			self.noise.get([block.x as f64 * self.scale, block.y as f64 * self.scale, block.z as f64 * self.scale]))
	}
}

pub struct WorldGenerator {
	height_noise: CachedNoise2D,
	biome_height_noise: CachedNoise2D,
	biome_temp_noise: CachedNoise2D,
	biome_precipitation_noise: CachedNoise2D,
}

impl WorldGenerator {
	pub fn new(seed: u32) -> Self {
		WorldGenerator {
			height_noise: CachedNoise2D::new(seed, 0.05),
			biome_height_noise: CachedNoise2D::new(seed + 1, 0.002),
			biome_temp_noise: CachedNoise2D::new(seed + 2, 0.0002),
			biome_precipitation_noise: CachedNoise2D::new(seed + 3, 0.0002),
		}
	}

	fn get_height_noise(&self, block: BlockPos, amplitude: f64, cache: &mut NoiseCache) -> i64 {
		(amplitude * self.height_noise.get_block_pos(block, &mut cache.height_noise)) as i64
	}

	fn get_biome_height_noise(&self, block: BlockPos, cache: &mut NoiseCache) -> i64 {
		let noise = 6.0 * self.biome_height_noise.get_block_pos(block, &mut cache.biome_height_noise);
		(noise * noise * noise) as i64
	}

	fn get_biome_noise(&self, block: BlockPos, cache: &mut NoiseCache) -> BiomeNoiseData {
		// this seems to make it about uniform over the range of 0..16
		let temperature = (8.0 + (25.0 * self.biome_temp_noise.get_block_pos(block, &mut cache.biome_temp_noise))).clamp(0.0, 15.0) as u8;
		let precipitation = (8.0 + (25.0 * self.biome_precipitation_noise.get_block_pos(block, &mut cache.biome_precipitation_noise))).clamp(0.0, 15.0) as u8;
		BiomeNoiseData {
			temperature,
			precipitation,
		}
	}

	pub fn generate_chunk(&self, world: Arc<World>, position: ChunkPos) -> LoadedChunk {
		let mut cache = NoiseCache::default();
		LoadedChunk::new(Chunk::new(world, position, |block| {
			let biome_height = self.get_biome_height_noise(block, &mut cache);
			let biome_noise = self.get_biome_noise(block, &mut cache);

			let biome = SurfaceBiome::new(biome_noise);

			let height = self.get_height_noise(block, biome.height_amplitude(), &mut cache);

			biome.get_block_at_depth(block.y - height)
		}))
	}
}

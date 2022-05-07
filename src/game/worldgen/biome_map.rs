use noise::{Seedable, NoiseFn, OpenSimplex};
use petgraph::{graph::{Graph, NodeIndex}, Undirected};
use rustc_hash::{FxHashMap, FxHashSet};
use parking_lot::{RwLock, RwLockWriteGuard};

use crate::prelude::*;
use super::biome::*;

// represents output layers of noise
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiomeNoise(i64, i64);

// stores biomes and generates new biomes at a given position
pub struct BiomeMap {
	// uses 2 layers of noise to determine where biomes stop and start
	noise: (OpenSimplex, OpenSimplex),
	regions: RwLock<FxHashMap<BlockPos, NodeIndex>>,
	biome_graph: RwLock<Graph<Option<SurfaceBiome>, (), Undirected>>,
}

impl BiomeMap {
	// setting this higher will make smaller biomes on average
	const BIOME_NOISE_FREQUENCY: f64 = 0.001;
	const BIOME_NOISE_AMPLITUDE: f64 = 6.0;

	pub fn new(seed: u32) -> Self {
		Self {
			noise: (OpenSimplex::new().set_seed(seed),
				OpenSimplex::new().set_seed(seed + 7481)),
			regions: RwLock::new(FxHashMap::default()),
			biome_graph: RwLock::new(Graph::new_undirected()),
		}
	}

	pub fn get_biome(&self, block: BlockPos) -> SurfaceBiome {
		if let Some(biome_index) = self.regions.read().get(&block) {
			// panic safety: once we have read abilities on the lock
			// and the block has a corresponding biome in the graph,
			// then the biome would be some at this point
			self.biome_graph.read()[*biome_index].unwrap()
		} else {
			let mut regions = self.regions.write();

			let mut biome_graph = self.biome_graph.write();
			let biome_index = biome_graph.add_node(None);

			let biome_noise = self.get_biome_noise(block);
			let mut visited_blocks = FxHashSet::default();

			self.visit_neighbors(
				block,
				None,
				biome_index,
				biome_noise,
				&mut visited_blocks,
				&mut regions,
				&mut biome_graph
			);

			todo!();
		}
	}

	// TODO: mayber put this in a loop
	fn visit_neighbors(
		&self,
		block: BlockPos,
		visited_from: Option<BlockPos>,
		biome_index: NodeIndex,
		current_biome_noise: BiomeNoise,
		visited_blocks: &mut FxHashSet<BlockPos>,
		regions: &mut RwLockWriteGuard<FxHashMap<BlockPos, NodeIndex>>,
		biome_graph: &mut RwLockWriteGuard<Graph<Option<SurfaceBiome>, (), Undirected>>,
	) {
		if visited_blocks.contains(&block) {
			return;
		}
		visited_blocks.insert(block);
	
		// if the block is part of another biome, add an edge from the current biome to the other biome
		// since we know it has not been visited yet in the current pass,
		// but it has been visited in a different pass, we know the noise
		// is different so no need to check that
		if let Some(other_biome_index) = regions.get(&block) {
			biome_graph.update_edge(biome_index, *other_biome_index, ());
		} else if current_biome_noise == self.get_biome_noise(block) {
			regions.insert(block, biome_index);
	
			for visit_direction in [
				BlockPos::new(1, 0, 0),
				BlockPos::new(-1, 0, 0),
				BlockPos::new(0, 0, 1),
				BlockPos::new(0, 0, -1),
			].iter() {
				let new_block = block + visit_direction;
				if visited_from.is_none() || new_block != visited_from.unwrap() {
					self.visit_neighbors(
						new_block,
						Some(block),
						biome_index,
						current_biome_noise,
						visited_blocks,
						regions,
						biome_graph,
					);
				}
			}
		}
	}

	fn get_biome_noise(&self, block: BlockPos) -> BiomeNoise {
		BiomeNoise(
			(Self::BIOME_NOISE_AMPLITUDE * self.noise.0.get([
				block.x as f64 * Self::BIOME_NOISE_FREQUENCY,
				block.z as f64 * Self::BIOME_NOISE_FREQUENCY,
			])) as i64,
			(Self::BIOME_NOISE_AMPLITUDE * self.noise.1.get([
				block.x as f64 * Self::BIOME_NOISE_FREQUENCY,
				block.z as f64 * Self::BIOME_NOISE_FREQUENCY,
			])) as i64,
		)
	}
}

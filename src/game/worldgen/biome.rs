use crate::prelude::*;
use crate::game::block::{Block, Air, Grass, Dirt, RockyDirt, Stone};

#[derive(Debug, Clone, Copy)]
pub struct BiomeNoiseData {
	// there will be 16 different temperature and precipitation levels used to determine the biome type
	pub temperature: u8,
	pub precipitation: u8,
}

// the first index is temperature, the second is precipitation
const BIOME_MAP: [[SurfaceBiome; 16]; 16] = {
	use SurfaceBiome::*;
	// colder
	[
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		// drier																																												wetter
	]
	// hotter
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceBiome {
	// oak an birch trees grow here
	Grasslands,
	// grasslands with far more frequent vegetation and shrubs
	LushGrasslands,
	// coniferous tree is a tree like a pine tree
	ConiferousForest,
	// forest of other trees
	BroadleafForest,
	// very moist forest
	Jungle,
	Swamp,
	// like a swamp but with no trees
	FloodedGrasslands,
	// dry, cold, little vegetation
	Tundra,
	// ice and snow evrywhere
	Arctic,
	// a cold forest
	Taiga,
	Desert,
	// like a desert but with more shrubs growing and vegetation
	XericShrubland,
	// savanna with very few trees, mostly grass
	SavannaGrassland,
	// savanna with much more frequent trees
	SavannaWoodland,
}

impl SurfaceBiome {
	pub fn new(biome_noise: BiomeNoiseData) -> Self {
		BIOME_MAP[biome_noise.temperature as usize][biome_noise.precipitation as usize]
	}

	pub fn height_amplitude(&self) -> f64 {
		match self {
			Self::Grasslands => 4.0,
			_ => 1.0,
		}
	}

	// depth is negative for blocks below the surface, and 0 at the surface
	// returns none if the depth is too deep and it is not apart of this biome
	pub fn get_block_at_depth(&self, depth: i64) -> Box<dyn Block> {
		if depth > 0 {
			Air::new()
		} else {
			match self {
				Self::Grasslands => {
					// TODO: make a macro for this
					if depth == 0 {
						Grass::new()
					} else if depth > -3 {
						Dirt::new()
					} else if depth > -6 {
						RockyDirt::new()
					} else {
						Stone::new()
					}
				},
				_ => Stone::new(),
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountainBiome {
	SnowyPeaks,
	BarrenPeaks,
	MontaneForest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeaBiome {
	Sea,
	FrozenSea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndergroundBiome {
	SolidGround,
	Caverns,
	LushCaverns,
	UndergroundLake,
	FloodedCaverns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderworldBiome {
}

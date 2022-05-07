use crate::prelude::*;

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
	pub fn new(temp: i64, precipitation: i64) -> Self {
		// just return grasslands for now
		Self::Grasslands
	}

	pub fn height_amplitude(&self) -> f64 {
		match self {
			Self::Grasslands => 4.0,
			_ => 1.0,
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

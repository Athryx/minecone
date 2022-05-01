use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceBiome {
	GrassyPlains,
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
	SavannaWoodland,
	// savanna with much more frequent trees
	SavannaGrassland,
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

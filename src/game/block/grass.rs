use super::*;

pub struct Grass {}

impl Grass {
	pub fn new() -> Grass {
		Grass {}
	}
}

impl BlockTrait for Grass {
	fn name(&self) -> &str {
		"grass"
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Grass
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

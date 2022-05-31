use super::*;

pub struct Dirt {}

impl Dirt {
	pub fn new() -> Dirt {
		Dirt {}
	}
}

impl BlockTrait for Dirt {
	fn name(&self) -> &str {
		"dirt"
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Dirt
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

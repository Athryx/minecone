use super::*;

pub struct Stone {}

impl Stone {
	pub fn new() -> Stone {
		Stone {}
	}
}

impl BlockTrait for Stone {
	fn name(&self) -> &str {
		"stone"
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Stone
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

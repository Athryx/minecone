use super::*;

pub struct RockyDirt {}

impl RockyDirt {
	pub fn new() -> RockyDirt {
		RockyDirt {}
	}
}

impl BlockTrait for RockyDirt {
	fn name(&self) -> &str {
		"rocky dirt"
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::RockyDirt
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

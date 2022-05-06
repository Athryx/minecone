use super::*;

pub struct RockyDirt {}

impl RockyDirt {
	pub fn new() -> Box<dyn Block> {
		Box::new(RockyDirt {})
	}
}

impl Block for RockyDirt {
	fn name(&self) -> &str {
		"rocky dirt"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::RockyDirt
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::RockyDirt
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

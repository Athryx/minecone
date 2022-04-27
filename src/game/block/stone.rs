use super::*;

pub struct Stone {}

impl Stone {
	pub fn new() -> Box<dyn Block> {
		Box::new(Stone {})
	}
}

impl Block for Stone {
	fn name(&self) -> &str {
		"stone"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::Stone
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Stone
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

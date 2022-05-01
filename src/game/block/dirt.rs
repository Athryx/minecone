use super::*;

pub struct Dirt {}

impl Dirt {
	pub fn new() -> Box<dyn Block> {
		Box::new(Dirt {})
	}
}

impl Block for Dirt {
	fn name(&self) -> &str {
		"dirt"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::Dirt
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Dirt
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

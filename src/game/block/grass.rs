use super::*;

pub struct Grass {}

impl Grass {
	pub fn new() -> Box<dyn Block> {
		Box::new(Grass {})
	}
}

impl Block for Grass {
	fn name(&self) -> &str {
		"dirt"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::Grass
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::Grass
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

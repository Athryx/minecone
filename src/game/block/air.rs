use super::*;

pub struct Air {}

impl Air {
	pub fn new() -> Box<dyn Block> {
		Box::new(Air {})
	}
}

impl Block for Air {
	fn name(&self) -> &str {
	    "air"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::Air
	}

	fn texture_index(&self) -> TextureIndex {
	    panic!("tried to get the TextureType of air");
	}

	fn is_translucent(&self) -> bool {
		true
	}
}

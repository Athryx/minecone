use super::*;

pub struct Air {}

impl Air {
	pub fn new() -> Air {
		Air {}
	}
}

impl BlockTrait for Air {
	fn name(&self) -> &str {
	    "air"
	}

	fn texture_index(&self) -> TextureIndex {
	    panic!("tried to get the TextureType of air");
	}

	fn is_translucent(&self) -> bool {
		true
	}
}

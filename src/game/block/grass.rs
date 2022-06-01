use super::*;

pub struct Grass {}

impl Grass {
	pub fn new() -> Grass {
		Grass {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		loader().load_image("textures/grass.png")
	}
}

impl BlockTrait for Grass {
	fn name(&self) -> &str {
		"grass"
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

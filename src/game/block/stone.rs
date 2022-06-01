use super::*;

pub struct Stone {}

impl Stone {
	pub fn new() -> Stone {
		Stone {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		loader().load_image("textures/stone.png")
	}
}

impl BlockTrait for Stone {
	fn name(&self) -> &str {
		"stone"
	}

	fn is_translucent(&self) -> bool {
		false
	}
}

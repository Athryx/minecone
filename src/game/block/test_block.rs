use super::*;

pub struct TestBlock {}

impl TestBlock {
	pub fn new() -> TestBlock {
		TestBlock {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		loader().load_image("textures/test-block.png")
	}
}

impl BlockTrait for TestBlock {
	fn name(&self) -> &str {
		"test block"
	}

	fn is_translucent(&self) -> bool {
		// it is not translucent, but we want to be able to see the test block everywhere it is for testing purposes
		true
	}
}

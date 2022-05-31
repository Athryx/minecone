use super::*;

pub struct TestBlock {}

impl TestBlock {
	pub fn new() -> TestBlock {
		TestBlock {}
	}
}

impl BlockTrait for TestBlock {
	fn name(&self) -> &str {
		"test block"
	}

	fn texture_index(&self) -> TextureIndex {
		TextureIndex::TestBlock
	}

	fn is_translucent(&self) -> bool {
		// it is not translucent, but we want to be able to see the test block everywhere it is for testing purposes
		true
	}
}

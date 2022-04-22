use super::*;

static TEST_BLOCK_MODEL: BlockModel = BlockModel::from_texture(
	TextureFace::Up(
		TextureSegment::from_tl(TexPos::new(0.0, 1.0))
	)
);

pub struct TestBlock {}

impl TestBlock {
	pub fn new() -> Box<dyn Block> {
		Box::new(TestBlock {})
	}
}

impl Block for TestBlock {
	fn name(&self) -> &str {
		"test block"
	}

	fn block_type(&self) -> BlockType {
	    BlockType::TestBlock
	}

	fn model(&self) -> &'static BlockModel {
		&TEST_BLOCK_MODEL
	}

	fn is_translucent(&self) -> bool {
		// it is not translucent, but we want to be able to see the test block everywhere it is for testing purposes
		true
	}
}

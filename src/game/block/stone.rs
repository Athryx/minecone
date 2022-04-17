use super::*;

static STONE_MODEL: BlockModel = BlockModel::from_texture(
	TextureFace::Up(
		TextureSegment::from_tl(TexPos::new(0.0, 0.0))
	)
);

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

	fn model(&self) -> &'static BlockModel {
		&STONE_MODEL
	}
}

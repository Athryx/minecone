use super::*;

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

	fn model_path(&self) -> &Path {
		"blocks/stone.obj".as_ref()
	}
}

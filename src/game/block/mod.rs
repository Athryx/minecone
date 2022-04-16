use std::path::Path;

use nalgebra::Vector3;

pub use crate::render::model::Model;

mod stone;
pub use stone::*;

pub type BlockPos = Vector3<f64>;

pub enum BlockType {
	Stone,
}

pub trait Block {
	fn name(&self) -> &str;
	fn block_type(&self) -> BlockType;
	fn model_path(&self) -> &Path;
}

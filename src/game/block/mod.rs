use std::path::Path;

use nalgebra::Vector3;

mod stone;
pub use stone::*;

pub type BlockPos = Vector3<f64>;

pub trait Block {
	fn name(&self) -> &str;
	fn model_path(&self) -> &Path;
}

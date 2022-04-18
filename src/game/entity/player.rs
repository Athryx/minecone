use nalgebra::Vector3;

use super::*;

pub struct Player {
	position: Vector3<f64>,
}

impl Player {
	pub fn new(position: Vector3<f64>) -> Box<dyn Entity> {
		Box::new(Player {
			position
		})
	}
}

impl Entity for Player {
}

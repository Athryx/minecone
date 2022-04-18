use nalgebra::Vector3;

use super::*;
use crate::prelude::*;

pub struct Player {
	position: Position,
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

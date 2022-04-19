use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;

use winit::{
	window::Window,
	event::*,
	dpi::PhysicalSize
};

use crate::render::Renderer;
use crate::render::model::{Mesh, Material, ModelVertex};
use camera_controller::CameraController;
use super::world::World;
use super::block::BlockFace;

mod camera_controller;

pub struct Client {
	world: Rc<RefCell<World>>,
	world_mesh: Mesh,
	texture_map: Material,
	camera_controller: CameraController,
	renderer: Renderer,
}

impl Client {
	pub fn new(window: &Window, world: Rc<RefCell<World>>) -> Self {
		let renderer = pollster::block_on(Renderer::new(window));

		let texture_map = Material::load_from_file("texture-map.png", "texture map", renderer.context())
			.expect("could not load texture map");

		let mut vertexes = Vec::new();
		let mut indexes = Vec::new();

		let mut current_index = 0;
		for block_face in world.borrow().world_mesh() {
			vertexes.extend(block_face.0.iter().map(|elem| Into::<ModelVertex>::into(*elem)));
			indexes.extend(BlockFace::indicies().iter().map(|elem| elem + current_index));
			current_index += 4;
		}

		let mesh = Mesh::new(
			"world mesh",
			&vertexes,
			&indexes,
			0,
			renderer.context()
		);

		Self {
			world,
			world_mesh: mesh,
			texture_map,
			camera_controller: CameraController::new(7.0, 20.0, 2.0),
			renderer,
		}
	}

	pub fn input(&mut self, event: &WindowEvent) {
		self.camera_controller.process_event(event);
	}

	pub fn frame_update(&mut self, new_window_size: Option<PhysicalSize<u32>>) {
		if let Some(new_window_size) = new_window_size {
			self.renderer.resize(new_window_size);
		}
		self.renderer.render(&[(&self.world_mesh, &self.texture_map)]);
	}

	pub fn physics_update(&mut self, delta: Duration) {
		self.camera_controller.update_camera(self.renderer.get_camera_mut(), delta);
		self.renderer.render(&[(&self.world_mesh, &self.texture_map)]);
	}
}
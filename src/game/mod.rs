use std::time::{Instant, Duration};

use winit::window::WindowId;
use winit::{window::Window, event::*, event_loop::ControlFlow};

use crate::render::Renderer;
use camera_controller::CameraController;
use world::World;

mod camera_controller;
mod entity;
mod block;
mod chunk;
mod world;

pub struct Game {
	window_id: WindowId,
	renderer: Renderer,
	camera_controller: CameraController,
	frame_time: Duration,
	last_update_time: Instant,
	world: World,
}

impl Game {
	pub fn new(framerate: u64, window: & Window) -> Self {
		let frame_time = Duration::from_micros(1_000_000 / framerate);
		Game {
			window_id: window.id(),
			renderer: pollster::block_on(Renderer::new(window)),
			camera_controller: CameraController::new(2.0),
			frame_time,
			last_update_time: Instant::now() - frame_time,
			world: World::new_test().expect("could not load the test world"),
		}
	}

	pub fn input(&mut self, event: &WindowEvent) {
		self.camera_controller.process_event(&event);
	}

	pub fn physics_update(&mut self) -> ControlFlow {
		let current_time = Instant::now();
		let time_delta = current_time - self.last_update_time;

		if time_delta > self.frame_time {
			self.camera_controller.update_camera(self.renderer.get_camera_mut(), time_delta);
			self.renderer.render();
			self.last_update_time = current_time;
		}
		ControlFlow::WaitUntil(self.last_update_time + self.frame_time)
	}

	pub fn event_update(&mut self, event: Event<()>) -> ControlFlow {
		match event {
			Event::RedrawRequested(window_id) if window_id == self.window_id => {
				self.renderer.render();
				self.physics_update()
			},
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == self.window_id => {
				match event {
					WindowEvent::CloseRequested
					| WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								state: ElementState::Pressed,
								virtual_keycode: Some(VirtualKeyCode::Escape),
								..
							},
						..
					} => return ControlFlow::Exit,
					WindowEvent::Resized(new_size) => self.renderer.resize(*new_size),
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.renderer.resize(**new_inner_size),
					_ => self.input(event),
				}
				self.physics_update()
			},
			_ => self.physics_update(),
		}
	}
}

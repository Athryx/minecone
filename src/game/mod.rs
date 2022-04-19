use std::time::{Instant, Duration};
use std::cell::RefCell;
use std::rc::Rc;

use winit::window::WindowId;
use winit::{
	window::Window,
	event::*,
	event_loop::ControlFlow,
	dpi::PhysicalSize,
};

use world::World;
use client::Client;

mod client;
mod entity;
mod block;
mod chunk;
mod world;
mod worldgen;

// Game is in charge of calling frame_update and physics_update on the correct intervals
// and dispatching input events
pub struct Game {
	window_id: WindowId,
	frame_time: Duration,
	last_update_time: Instant,
	world: Rc<RefCell<World>>,
	client: Client,
}

impl Game {
	pub fn new(framerate: u64, window: &Window) -> Self {
		let frame_time = Duration::from_micros(1_000_000 / framerate);

		let world = World::new_test().expect("could not load the test world");
		let client = Client::new(window, world.clone());

		Self {
			window_id: window.id(),
			frame_time,
			last_update_time: Instant::now() - frame_time,
			world,
			client,
		}
	}

	pub fn input(&mut self, event: &WindowEvent) {
		self.client.input(event);
	}

	// TODO: implement correctly, with redrawing every so often
	pub fn frame_update(&mut self, new_window_size: Option<PhysicalSize<u32>>) {
		self.client.frame_update(new_window_size);
	}

	pub fn try_physics_update(&mut self) -> ControlFlow {
		let current_time = Instant::now();
		let time_delta = current_time - self.last_update_time;

		if time_delta > self.frame_time {
			self.client.physics_update(time_delta);
			self.last_update_time = current_time;
		}
		ControlFlow::WaitUntil(self.last_update_time + self.frame_time)
	}

	pub fn event_update(&mut self, event: Event<()>) -> ControlFlow {
		match event {
			Event::RedrawRequested(window_id) if window_id == self.window_id => {
				self.frame_update(None);
				self.try_physics_update()
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
					WindowEvent::Resized(new_size) => self.frame_update(Some(*new_size)),
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.frame_update(Some(**new_inner_size)),
					_ => self.input(event),
				}
				self.try_physics_update()
			},
			_ => self.try_physics_update(),
		}
	}
}

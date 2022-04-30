#![feature(once_cell)]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(let_chains)]
#![feature(test)]

#![warn(clippy::disallowed_types)]

#[macro_use]
extern crate log;

use winit::{
	event_loop::EventLoop,
	window::WindowBuilder,
	dpi::PhysicalSize,
};

mod assets;
mod render;
mod game;
mod util;
mod prelude;

fn main() {
	pretty_env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("Mineclone")
		.with_inner_size(PhysicalSize::new(1280, 720))
		.build(&event_loop)
		.unwrap();

	let mut game = game::Game::new(60, &window);

	event_loop.run(move |event, _, control_flow| {
		*control_flow = game.event_update(event);
	});
}

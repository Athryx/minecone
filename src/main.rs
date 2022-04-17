#![feature(once_cell)]
#![feature(const_fn_floating_point_arithmetic)]

#[macro_use]
extern crate log;

use winit::{
	event_loop::EventLoop,
	window::WindowBuilder,
};

mod assets;
mod render;
mod game;
mod util;

fn main() {
	pretty_env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("Mineclone")
		.build(&event_loop)
		.unwrap();

	let mut game = game::Game::new(60, &window);

	event_loop.run(move |event, _, control_flow| {
		*control_flow = game.event_update(event);
	});
}

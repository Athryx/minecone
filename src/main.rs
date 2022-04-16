#![feature(once_cell)]

#[macro_use]
extern crate log;

extern crate nalgebra_glm as glm;

use winit::{
	event_loop::EventLoop,
	window::WindowBuilder,
};

mod assets;
mod render;
mod game;
mod world;
mod camera_controller;

pub fn run() {
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

fn main() {
	pretty_env_logger::init();
	run()
}

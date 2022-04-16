#![feature(once_cell)]

#[macro_use]
extern crate log;

extern crate nalgebra_glm as glm;

use std::time::{Instant, Duration};

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

mod assets;
mod render;

pub async fn run() {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("Mineclone")
		.build(&event_loop)
		.unwrap();

	let mut state = render::State::new(&window).await;

	let frame_time = Duration::from_millis(17);
	let mut last_update_time = Instant::now();

	event_loop.run(move |event, _, control_flow| {
		let current_time = Instant::now();
		let update_time_delta = current_time - last_update_time;

		let mut update = |state: &mut render::State, control_flow: &mut ControlFlow| {
			if update_time_delta > frame_time {
				state.update(update_time_delta);
				match state.render() {
					Ok(_) => (),
					// reconfigure surface if lost
					Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
					Err(wgpu::SurfaceError::OutOfMemory) => {
						error!("out of memory");
						*control_flow = ControlFlow::Exit;
					}
					Err(e) => warn!("{:?}", e),
				}
				last_update_time = current_time;
			}
		};

		match event {
			Event::RedrawRequested(window_id) if window_id == window.id() => {
				update(&mut state, control_flow);
			}
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == window.id() => if !state.input(event) {
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
					} => *control_flow = ControlFlow::Exit,
					WindowEvent::Resized(new_size) => state.resize(*new_size),
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
					_ => {}
				}
				update(&mut state, control_flow);
			},
			_ => update(&mut state, control_flow),
		}

		if *control_flow != ControlFlow::Exit {
			*control_flow = ControlFlow::WaitUntil(last_update_time + frame_time);
		}
	});
}

fn main() {
	pretty_env_logger::init();
	pollster::block_on(run());
}

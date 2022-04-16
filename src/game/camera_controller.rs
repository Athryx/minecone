use std::time::Duration;

use winit::event::*;

use crate::render::camera::Camera;

#[derive(Debug)]
pub struct CameraController {
	speed: f32,
	is_forward_pressed: bool,
	is_backward_pressed: bool,
	is_left_pressed: bool,
	is_right_pressed: bool,
}

impl CameraController {
	pub fn new(speed: f32) -> Self {
		Self {
			speed,
			is_forward_pressed: false,
			is_backward_pressed: false,
			is_left_pressed: false,
			is_right_pressed: false,
		}
	}

	pub fn process_event(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::KeyboardInput {
				input: KeyboardInput {
					state,
					virtual_keycode: Some(keycode),
					..
				},
				..
			} => {
				let is_pressed = *state == ElementState::Pressed;
				match keycode {
					VirtualKeyCode::W | VirtualKeyCode::Up => {
						self.is_forward_pressed = is_pressed;
						true
					}
					VirtualKeyCode::A | VirtualKeyCode::Left => {
						self.is_left_pressed = is_pressed;
						true
					}
					VirtualKeyCode::S | VirtualKeyCode::Down => {
						self.is_backward_pressed = is_pressed;
						true
					}
					VirtualKeyCode::D | VirtualKeyCode::Right => {
						self.is_right_pressed = is_pressed;
						true
					}
					_ => false,
				}
			}
			_ => false,
		}
	}

	pub fn update_camera(&self, camera: &mut Camera, time_delta: Duration) {
		let forward = camera.look_at - camera.position;
		let forward_norm = forward.normalize();
		let forward_mag = forward.magnitude();

		let distance_moved = time_delta.as_millis() as f32 * self.speed / 1000.0;

		// Prevents glitching when camera gets too close to the
		// center of the scene.
		if self.is_forward_pressed && forward_mag > distance_moved {
			camera.position += forward_norm * distance_moved;
		}
		if self.is_backward_pressed {
			camera.position -= forward_norm * distance_moved;
		}

		let right = forward_norm.cross(&camera.up);

		// Redo radius calc in case the fowrard/backward is pressed.
		let forward = camera.look_at - camera.position;
		let forward_mag = forward.magnitude();

		if self.is_right_pressed {
			// Rescale the distance between the target and eye so 
			// that it doesn't change. The eye therefore still 
			// lies on the circle made by the target and eye.
			camera.position = camera.look_at - (forward + right * distance_moved).normalize() * forward_mag;
		}
		if self.is_left_pressed {
			camera.position = camera.look_at - (forward - right * distance_moved).normalize() * forward_mag;
		}
	}
}

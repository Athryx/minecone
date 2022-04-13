use std::time::Duration;

use glm::{Vec3, Mat4};
use nalgebra::Point3;
use winit::event::*;

const TO_GPU_MATRIX: Mat4 = Mat4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.0,
	0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
	position: Point3<f32>,
	look_at: Point3<f32>,
	up: Vec3,
	aspect_ratio: f32,
	fovy: f32,
	znear: f32,
	zfar: f32,
}

impl Camera {
	pub fn new(position: Point3<f32>, look_at: Point3<f32>, aspect_ratio: f32) -> Self {
		Self {
			position,
			look_at,
			up: *Vec3::y_axis(),
			aspect_ratio,
			fovy: 45.0,
			znear: 0.1,
			zfar: 100.0,
		}
	}

	pub fn get_camera_matrix(&self) -> Mat4 {
		let view = Mat4::look_at_rh(&self.position, &self.look_at, &self.up);
		let proj = Mat4::new_perspective(self.aspect_ratio, self.fovy, self.znear, self.zfar);

		return TO_GPU_MATRIX * proj * view;
	}

	// gets a camera uniform which can be sent to the gpu
	pub fn get_camera_uniform(&self) -> CameraUniform {
		CameraUniform(self.get_camera_matrix().into())
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform([[f32; 4]; 4]);

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

	pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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

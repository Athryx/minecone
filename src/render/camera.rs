use glm::{Vec3, Mat4};
use nalgebra::Point3;

const TO_GPU_MATRIX: Mat4 = Mat4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.0,
	0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug)]
pub struct Camera {
	// these need to be public because camera controller modifies these
	pub position: Point3<f32>,
	pub look_at: Point3<f32>,
	pub up: Vec3,
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

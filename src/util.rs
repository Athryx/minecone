use nalgebra::{Vector2, Vector3};

#[macro_export]
macro_rules! array3d_init {
	($value:expr) => {
		array_init::array_init(|_| array_init::array_init(|_| array_init::array_init(|_| $value)))
	};
}

// get vector components in a const function context
pub const fn vec2_getx<T: Copy>(vector: Vector2<T>) -> T {
	vector.data.0[0][0]
}

pub const fn vec2_gety<T: Copy>(vector: Vector2<T>) -> T {
	vector.data.0[0][1]
}

pub const fn vec3_getx<T: Copy>(vector: Vector3<T>) -> T {
	vector.data.0[0][0]
}

pub const fn vec3_gety<T: Copy>(vector: Vector3<T>) -> T {
	vector.data.0[0][1]
}

pub const fn vec3_getz<T: Copy>(vector: Vector3<T>) -> T {
	vector.data.0[0][2]
}

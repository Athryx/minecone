// Vertex shader

struct CameraUniform {
	view_proj: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
	[[location(0)]] position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] texture_index: i32;
};

struct VertexOutput {
	[[builtin(position)]] clip_position: vec4<f32>;
	[[location(0)]] world_pos: vec3<f32>;
	[[location(1)]] world_normal: vec3<f32>;
	[[location(2)]] texture_index: i32;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
	out.world_pos = model.position;
	out.world_normal = model.normal;
	out.texture_index = model.texture_index;
	return out;
}


// Fragment shader

[[group(0), binding(0)]]
var block_diffuse_textures: texture_2d_array<f32>;
[[group(0), binding(1)]]
var block_diffuse_sampler: sampler;

fn wrap_pos(n: f32) -> f32 {
	if (n >= 0.0) {
		return n % 1.0;
	} else {
		return 1.0 - (-n % 1.0);
	}
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	var x_sample: f32;
	var y_sample: f32;

	if (in.world_normal.x != 0.0) {
		x_sample = in.world_pos.z;
		y_sample = in.world_pos.y;
	} else if (in.world_normal.y != 0.0) {
		x_sample = in.world_pos.x;
		y_sample = in.world_pos.z;
	} else {
		x_sample = in.world_pos.x;
		y_sample = in.world_pos.y;
	}

	// adjust this constant whenever texture map size is changed
	let sample_pos = vec2<f32>(wrap_pos(x_sample), wrap_pos(y_sample));
	return textureSample(block_diffuse_textures, block_diffuse_sampler, sample_pos, in.texture_index);
}

use std::mem;
use std::ops::Range;

use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::texture::Texture;
use crate::assets::loader;

pub trait Vertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
	normal: [f32; 3],
}

impl ModelVertex {
	const ATTRIBS: [wgpu::VertexAttribute; 3] =
		wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
}

impl Vertex for ModelVertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}

#[derive(Debug)]
pub struct Mesh {
	pub name: String,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub num_elements: u32,
	pub material_index: usize,
}

#[derive(Debug)]
pub struct Material {
	pub name: String,
	pub diffuse_texture: Texture,
	pub bind_group: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct Model {
	// temp
	pub meshes: Vec<Mesh>,
	materials: Vec<Material>,
}

impl Model {
	pub fn load_from_file(
		file_name: &str,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		layout: &wgpu::BindGroupLayout,
	) -> Result<Self> {
		let (obj_meshes, obj_materials) = loader().load_obj(file_name)?;

		let mut materials = Vec::with_capacity(obj_materials.len());
		for mat in obj_materials.into_iter() {
			let diffuse_texture = Texture::from_file(device, queue, &mat.diffuse_texture, &mat.diffuse_texture)?;
			let bind_group = device.create_bind_group(
				&wgpu::BindGroupDescriptor {
					label: Some(&format!("{} bind group", &mat.diffuse_texture)),
					layout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
						},
					],
				}
			);

			materials.push(Material {
				name: mat.diffuse_texture,
				diffuse_texture,
				bind_group,
			});
		}

		let mut meshes = Vec::with_capacity(obj_meshes.len());
		for mesh in obj_meshes.into_iter() {
			let vertices = (0..mesh.mesh.positions.len() / 3)
				.map(|i| ModelVertex {
					position: [
						mesh.mesh.positions[i * 3],
						mesh.mesh.positions[i * 3 + 1],
						mesh.mesh.positions[i * 3 + 2],
					],
					tex_coords: [
						mesh.mesh.texcoords[i * 2],
						mesh.mesh.texcoords[i * 2 + 1],
					],
					normal: [
						mesh.mesh.normals[i * 3],
						mesh.mesh.normals[i * 3 + 1],
						mesh.mesh.normals[i * 3 + 2],
					],
				})
				.collect::<Vec<_>>();

			let vertex_buffer = device.create_buffer_init(
				&wgpu::util::BufferInitDescriptor {
					label: Some(&format!("{} vertex buffer", &mesh.name)),
					contents: bytemuck::cast_slice(&vertices),
					usage: wgpu::BufferUsages::VERTEX,
				}
			);

			let index_buffer = device.create_buffer_init(
				&wgpu::util::BufferInitDescriptor {
					label: Some(&format!("{} index buffer", &mesh.name)),
					contents: bytemuck::cast_slice(&mesh.mesh.indices),
					usage: wgpu::BufferUsages::INDEX,
				}
			);

			meshes.push(Mesh {
				name: mesh.name,
				vertex_buffer,
				index_buffer,
				num_elements: mesh.mesh.indices.len().try_into().unwrap(),
				material_index: mesh.mesh.material_id.unwrap_or(0),
			});
		}

		Ok(Model {
			meshes,
			materials,
		})
	}
}

// model.rs
pub trait DrawModel<'a> {
	fn draw_mesh(&mut self, mesh: &'a Mesh);
	fn draw_mesh_instanced(
		&mut self,
		mesh: &'a Mesh,
		instances: Range<u32>,
	);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
	'b: 'a,
{
	fn draw_mesh(&mut self, mesh: &'b Mesh) {
		self.draw_mesh_instanced(mesh, 0..1);
	}

	fn draw_mesh_instanced(
		&mut self,
		mesh: &'b Mesh,
		instances: Range<u32>,
	) {
		self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
		self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		self.draw_indexed(0..mesh.num_elements, 0, instances);
	}
}

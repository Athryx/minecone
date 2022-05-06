use std::{iter::FusedIterator, mem};

use nalgebra::Vector3;

pub use crate::render::model::{Vertex, Model};
use crate::util::{vec3_getx, vec3_gety, vec3_getz};
use crate::prelude::*;

mod air;
pub use air::*;
mod dirt;
pub use dirt::*;
mod grass;
pub use grass::*;
mod stone;
pub use stone::*;
mod test_block;
pub use test_block::*;

// the amount of overlap between block verticies to stop rendering artifacts from occuring
//const BLOCK_MODEL_OVERLAP: f64 = 0.00001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFace {
	XPos = 0,
	XNeg = 1,
	YPos = 2,
	YNeg = 3,
	ZPos = 4,
	ZNeg = 5,
}

impl BlockFace {
	// iterate over all the faces
	// since blockface itself is an iterator since I am lazy to make a newtype, this exists just for clarity
	pub fn iter() -> BlockFaceIter {
		BlockFaceIter(Some(Self::XPos))
	}

	pub fn block_pos_offset(&self) -> BlockPos {
		match self {
			Self::XPos => BlockPos::new(1, 0, 0),
			Self::XNeg => BlockPos::new(-1, 0, 0),
			Self::YPos => BlockPos::new(0, 1, 0),
			Self::YNeg => BlockPos::new(0, -1, 0),
			Self::ZPos => BlockPos::new(0, 0, 1),
			Self::ZNeg => BlockPos::new(0, 0, -1),
		}
	}

	pub fn is_positive_face(&self) -> bool {
		matches!(self, Self::XPos | Self::YPos | Self::ZPos)
	}

	pub fn is_negative_face(&self) -> bool {
		matches!(self, Self::XNeg | Self::YNeg | Self::ZNeg)
	}
}

impl From<BlockFace> for usize {
	fn from(face: BlockFace) -> usize { 
		face as usize
	}
}

pub struct BlockFaceIter(Option<BlockFace>);

impl Iterator for BlockFaceIter {
	type Item = BlockFace;

	fn next(&mut self) -> Option<Self::Item> {
		match self.0? {
			BlockFace::XPos => {
				self.0 = Some(BlockFace::XNeg);
				Some(BlockFace::XPos)
			}
			BlockFace::XNeg => {
				self.0 = Some(BlockFace::YPos);
				Some(BlockFace::XNeg)
			}
			BlockFace::YPos => {
				self.0 = Some(BlockFace::YNeg);
				Some(BlockFace::YPos)
			}
			BlockFace::YNeg => {
				self.0 = Some(BlockFace::ZPos);
				Some(BlockFace::YNeg)
			}
			BlockFace::ZPos => {
				self.0 = Some(BlockFace::ZNeg);
				Some(BlockFace::ZPos)
			}
			BlockFace::ZNeg => {
				self.0 = None;
				Some(BlockFace::ZNeg)
			}
		}
	}
}

impl FusedIterator for BlockFaceIter {}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureIndex {
	TestBlock = 0,
	Stone = 1,
	Dirt = 2,
	Grass = 3,
}

impl TextureIndex {
	const TEXTURE_PATHS: [&'static str; 4] = [
		"textures/test-block.png",
		"textures/stone.png",
		"textures/dirt.png",
		"textures/grass.png",
	];

	pub const fn num_textures() -> u32 {
		Self::TEXTURE_PATHS.len() as u32
	}

	pub const fn resource_paths() -> &'static [&'static str] {
		&Self::TEXTURE_PATHS
	}
}

impl From<TextureIndex> for i32 {
	fn from(texture_type: TextureIndex) -> i32 {
		texture_type as i32
	}
}

#[derive(Debug, Clone, Copy)]
pub struct OcclusionCorners {
	pub tl: u8,
	pub tr: u8,
	pub bl: u8,
	pub br: u8,
}

impl OcclusionCorners {
	pub fn pos_corner(&self) -> u8 {
		self.tr
	}

	pub fn neg_corner(&self) -> u8 {
		self.bl
	}

	pub fn xpos_yneg_corner(&self) -> u8 {
		self.br
	}

	pub fn xneg_ypos_corner(&self) -> u8 {
		self.tl
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
	position: [f32; 3],
	normal: [f32; 3],
	// texture color will be mutiplied by this color
	color: [f32; 3],
	// the wgpu sample function takes in a signed integer so we use it here
	texture_index: i32,
}

impl BlockVertex {
	// panics on invalid occlusion level
	pub fn new(position: Position, normal: Vector3<f32>, texture_index: TextureIndex, occlusion_level: u8) -> Self {
		Self {
			position: [position.x as f32, position.y as f32, position.z as f32],
			normal: [normal.x, normal.y, normal.z],
			color: match occlusion_level {
				0 => [1.0, 1.0, 1.0],
				1 => [0.8, 0.8, 0.8],
				2 => [0.6, 0.6, 0.6],
				3 => [0.4, 0.4, 0.4],
				_ => panic!("invalid occlusion level passed to BlockVertex::new()"),
			},
			texture_index: texture_index.into(),
		}
	}

	const ATTRIBS: [wgpu::VertexAttribute; 4] =
		wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Sint32];
}

impl Vertex for BlockVertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}

// the front of the face is the side from which the vertexes are going in a clockwise direction
// all the BlockVertexes must also be coplanar
#[derive(Debug, Clone, Copy)]
pub struct BlockFaceMesh(pub [BlockVertex; 4]);

impl BlockFaceMesh {
	// TODO: add small overlap on edges to stop rendering artifacts
	// occlusion levels in the array are: [tl, bl, br, tr]
	pub fn from_corners(face: BlockFace, texture_index: TextureIndex, tl_corner_block: BlockPos, br_corner_block: BlockPos, occlusion_data: OcclusionCorners) -> Self {
		let tl_corner_pos = tl_corner_block.as_position();
		let br_corner_pos = br_corner_block.as_position();

		let (tl_corner, br_corner) = match face {
			BlockFace::XPos => (
				tl_corner_pos + Vector3::new(1.0, 1.0, 0.0),
				br_corner_pos + Vector3::new(1.0, 0.0, 1.0),
			),
			BlockFace::XNeg => (
				tl_corner_pos + Vector3::new(0.0, 1.0, 1.0),
				br_corner_pos + Vector3::new(0.0, 0.0, 0.0),
			),
			BlockFace::YPos => (
				tl_corner_pos + Vector3::new(0.0, 1.0, 1.0),
				br_corner_pos + Vector3::new(1.0, 1.0, 0.0),
			),
			BlockFace::YNeg => (
				tl_corner_pos + Vector3::new(0.0, 0.0, 0.0),
				br_corner_pos + Vector3::new(1.0, 0.0, 1.0),
			),
			BlockFace::ZPos => (
				tl_corner_pos + Vector3::new(1.0, 1.0, 1.0),
				br_corner_pos + Vector3::new(0.0, 0.0, 1.0),
			),
			BlockFace::ZNeg => (
				tl_corner_pos + Vector3::new(0.0, 1.0, 0.0),
				br_corner_pos + Vector3::new(1.0, 0.0, 0.0),
			),
		};

		let (bl_corner, tr_corner) = match face {
			BlockFace::XPos | BlockFace::XNeg => (
				Position::new(vec3_getx(tl_corner), vec3_gety(br_corner), vec3_getz(tl_corner)),
				Position::new(vec3_getx(tl_corner), vec3_gety(tl_corner), vec3_getz(br_corner)),
			),
			BlockFace::YPos | BlockFace::YNeg => (
				Position::new(vec3_getx(tl_corner), vec3_gety(tl_corner), vec3_getz(br_corner)),
				Position::new(vec3_getx(br_corner), vec3_gety(tl_corner), vec3_getz(tl_corner)),
			),
			BlockFace::ZPos | BlockFace::ZNeg => (
				Position::new(vec3_getx(tl_corner), vec3_gety(br_corner), vec3_getz(tl_corner)),
				Position::new(vec3_getx(br_corner), vec3_gety(tl_corner), vec3_getz(tl_corner)),
			),
		};

		let normal = match face {
			BlockFace::XPos => Vector3::new(1.0, 0.0, 0.0),
			BlockFace::XNeg => Vector3::new(-1.0, 0.0, 0.0),
			BlockFace::YPos => Vector3::new(0.0, 1.0, 0.0),
			BlockFace::YNeg => Vector3::new(0.0, -1.0, 0.0),
			BlockFace::ZPos => Vector3::new(0.0, 0.0, 1.0),
			BlockFace::ZNeg => Vector3::new(0.0, 0.0, -1.0),
		};

		Self([
			 BlockVertex::new(tl_corner, normal, texture_index, occlusion_data.tl),
			 BlockVertex::new(bl_corner, normal, texture_index, occlusion_data.bl),
			 BlockVertex::new(br_corner, normal, texture_index, occlusion_data.br),
			 BlockVertex::new(tr_corner, normal, texture_index, occlusion_data.tr),
		])
	}

	// TODO: this is probably more complicated than it needs to be
	pub fn from_cube_corners(face: BlockFace, texture_index: TextureIndex, neg_corner_block: BlockPos, pos_corner_block: BlockPos, occlusion_data: OcclusionCorners) -> Self {
		let (tl_corner, br_corner, occlusion_data) = match face {
			BlockFace::XPos => (
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, pos_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.xpos_yneg_corner(),
					tr: occlusion_data.pos_corner(),
					bl: occlusion_data.neg_corner(),
					br: occlusion_data.xneg_ypos_corner(),
				},
			),
			BlockFace::XNeg => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, neg_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.pos_corner(),
					tr: occlusion_data.xpos_yneg_corner(),
					bl: occlusion_data.xneg_ypos_corner(),
					br: occlusion_data.neg_corner(),
				},
			),
			BlockFace::YPos => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, neg_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.xneg_ypos_corner(),
					tr: occlusion_data.pos_corner(),
					bl: occlusion_data.neg_corner(),
					br: occlusion_data.xpos_yneg_corner(),
				}
			),
			BlockFace::YNeg => (
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, pos_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.neg_corner(),
					tr: occlusion_data.xpos_yneg_corner(),
					bl: occlusion_data.xneg_ypos_corner(),
					br: occlusion_data.pos_corner(),
				}
			),
			BlockFace::ZPos => (
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, pos_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.pos_corner(),
					tr: occlusion_data.xneg_ypos_corner(),
					bl: occlusion_data.xpos_yneg_corner(),
					br: occlusion_data.neg_corner(),
				},
			),
			BlockFace::ZNeg => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, neg_corner_block.z),
				OcclusionCorners {
					tl: occlusion_data.xneg_ypos_corner(),
					tr: occlusion_data.pos_corner(),
					bl: occlusion_data.neg_corner(),
					br: occlusion_data.xpos_yneg_corner(),
				},
			),
		};

		Self::from_corners(face, texture_index, tl_corner, br_corner, occlusion_data)
	}

	// returns the indicies of the block model to be used for the index buffer
	pub const fn indicies() -> &'static [u32] {
		&[0, 2, 1, 2, 0, 3]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
	Air,
	TestBlock,
	Dirt,
	Grass,
	Stone,
}

pub trait Block: Send + Sync {
	fn name(&self) -> &str;
	fn block_type(&self) -> BlockType;
	// panics if the block is air (or some other block without a blockmodel)
	fn texture_index(&self) -> TextureIndex;
	fn is_translucent(&self) -> bool;

	fn is_air(&self) -> bool {
		self.block_type() == BlockType::Air
	}
}

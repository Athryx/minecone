use std::{iter::FusedIterator, path::Path, mem};

use nalgebra::{Vector2, Vector3};

pub use crate::render::model::{Vertex, Model};
use crate::{util::{vec2_getx, vec2_gety, vec3_getx, vec3_gety, vec3_getz}, render::model::ModelVertex};
use crate::prelude::*;

mod air;
pub use air::*;
mod stone;
pub use stone::*;
mod test_block;
pub use test_block::*;

pub type TexPos = Vector2<f64>;

// the width and height of the texture map in number of blocks
const TEX_MAP_BLOCK_WIDTH: f64 = 32.0;
const TEX_MAP_BLOCK_HEIGHT: f64 = 32.0;

// the amount of overlap between block verticies to stop rendering artifacts from occuring
//const BLOCK_MODEL_OVERLAP: f64 = 0.00001;

// offset from the edge of the texture that the texture view will be
// this causes the texture segment to be smaller than the actual texture by a little bit
// to avoid rendering artifacts
//const TEX_OFFSET: f64 = 0.0001;

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
}

static TEXTURE_PATHS: [&'static str; 2] = [
	"textures/stone.png",
	"textures/test-block.png",
];

impl TextureIndex {
	pub const COUNT: u32 = 2;

	pub fn resource_paths() -> &'static [&'static str] {
		&TEXTURE_PATHS
	}
}

impl From<TextureIndex> for i32 {
	fn from(texture_type: TextureIndex) -> i32 {
		texture_type as i32
	}
}


#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
	position: [f32; 3],
	normal: [f32; 3],
	// the wgpu sample function takes in a signed integer so we use it here
	texture_index: i32,
}

impl BlockVertex {
	pub fn new(position: Position, normal: Vector3<f32>, texture_index: TextureIndex) -> Self {
		Self {
			position: [position.x as f32, position.y as f32, position.z as f32],
			normal: [normal.x, normal.y, normal.z],
			texture_index: texture_index.into(),
		}
	}

	const ATTRIBS: [wgpu::VertexAttribute; 3] =
		wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Sint32];
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
	pub fn from_corners(face: BlockFace, texture_index: TextureIndex, tl_corner_block: BlockPos, br_corner_block: BlockPos) -> Self {
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
			 BlockVertex::new(tl_corner, normal, texture_index),
			 BlockVertex::new(bl_corner, normal, texture_index),
			 BlockVertex::new(br_corner, normal, texture_index),
			 BlockVertex::new(tr_corner, normal, texture_index),
		])
	}

	// TODO: this is probably more complicated than it needs to be
	pub fn from_cube_corners(face: BlockFace, texture_index: TextureIndex, neg_corner_block: BlockPos, pos_corner_block: BlockPos) -> Self {
		let (tl_corner, br_corner) = match face {
			BlockFace::XPos => (
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, pos_corner_block.z),
			),
			BlockFace::XNeg => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, neg_corner_block.z),
			),
			BlockFace::YPos => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, neg_corner_block.z),
			),
			BlockFace::YNeg => (
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, pos_corner_block.z),
			),
			BlockFace::ZPos => (
				BlockPos::new(pos_corner_block.x, pos_corner_block.y, pos_corner_block.z),
				BlockPos::new(neg_corner_block.x, neg_corner_block.y, pos_corner_block.z),
			),
			BlockFace::ZNeg => (
				BlockPos::new(neg_corner_block.x, pos_corner_block.y, neg_corner_block.z),
				BlockPos::new(pos_corner_block.x, neg_corner_block.y, neg_corner_block.z),
			),
		};

		Self::from_corners(face, texture_index, tl_corner, br_corner)
	}

	// returns the indicies of the block model to be used for the index buffer
	pub const fn indicies() -> &'static [u32] {
		&[0, 2, 1, 2, 0, 3]
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
	Air,
	Stone,
	TestBlock,
}

pub trait Block {
	fn name(&self) -> &str;
	fn block_type(&self) -> BlockType;
	// panics if the block is air (or some other block without a blockmodel)
	fn texture_index(&self) -> TextureIndex;
	fn is_translucent(&self) -> bool;

	fn is_air(&self) -> bool {
		self.block_type() == BlockType::Air
	}
}

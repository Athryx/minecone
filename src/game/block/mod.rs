use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::iter::FusedIterator;

use nalgebra::{Vector2, Vector3, Translation3, Point3};

pub use crate::render::model::Model;
use crate::{util::{vec2_getx, vec2_gety, vec3_getx, vec3_gety, vec3_getz}, render::model::ModelVertex};
use crate::prelude::*;
use super::chunk::LoadedChunk;
use super::world::World;

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
const BLOCK_MODEL_OVERLAP: f64 = 0.00001;

// offset from the edge of the texture that the texture view will be
// this causes the texture segment to be smaller than the actual texture by a little bit
// to avoid rendering artifacts
const TEX_OFFSET: f64 = 0.0001;

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

// scales the input tex_pos where 1 unit is 1 block to be in the correct coordinates for the gpu
const fn scale_tex_pos(tex_pos: TexPos) -> TexPos {
	TexPos::new(vec2_getx(tex_pos) / TEX_MAP_BLOCK_WIDTH, vec2_gety(tex_pos) / TEX_MAP_BLOCK_HEIGHT)
}

// constant conversion from block pos to Vector3<f32>
const fn block_pos_to_position(block: BlockPos) -> Position {
	Vector3::new(vec3_getx(block) as f64, vec3_gety(block) as f64, vec3_getz(block) as f64)
}

// constant way to add Vector3<f32>
const fn vec_add(a: Position, b: Position) -> Position {
	Position::new(
		vec3_getx(a) + vec3_getx(b),
		vec3_gety(a) + vec3_gety(b),
		vec3_getz(a) + vec3_getz(b),
	)
}

#[derive(Debug, Clone, Copy)]
pub struct BlockVertex {
	position: Position,
	tex_coord: TexPos,
}

impl BlockVertex {
	pub const fn new(position: Position, tex_coord: TexPos) -> Self {
		Self {
			position,
			tex_coord,
		}
	}
}

impl From<BlockVertex> for ModelVertex {
	fn from(vertex: BlockVertex) -> ModelVertex {
		ModelVertex {
			position: [vertex.position.x as f32, vertex.position.y as f32, vertex.position.z as f32],
			tex_coords: [vertex.tex_coord.x as f32, vertex.tex_coord.y as f32],
			normal: [0.0, 0.0, 0.0],
		}
	}
}

// the front of the face is the side from which the vertexes are going in a clockwise direction
// all the BlockVertexes must also be coplanar
#[derive(Debug, Clone, Copy)]
pub struct BlockFaceMesh(pub [BlockVertex; 4]);

impl BlockFaceMesh {
	const fn new(face: BlockFace, segment: TextureSegment) -> Self {
		Self::from_corners(face, segment, BlockPos::new(0, 0, 0), BlockPos::new(0, 0, 0))
	}

	// TODO: add small overlap on edges to stop rendering artifacts
	pub const fn from_corners(face: BlockFace, segment: TextureSegment, tl_corner_block: BlockPos, br_corner_block: BlockPos) -> Self {
		let tl_corner_pos = block_pos_to_position(tl_corner_block);
		let br_corner_pos = block_pos_to_position(br_corner_block);

		let (tl_corner, br_corner) = match face {
			BlockFace::XPos => (
				vec_add(tl_corner_pos, Vector3::new(1.0, 1.0, 0.0)),
				vec_add(br_corner_pos, Vector3::new(1.0, 0.0, 1.0)),
			),
			BlockFace::XNeg => (
				vec_add(tl_corner_pos, Vector3::new(0.0, 1.0, 1.0)),
				vec_add(br_corner_pos, Vector3::new(0.0, 0.0, 0.0)),
			),
			BlockFace::YPos => (
				vec_add(tl_corner_pos, Vector3::new(0.0, 1.0, 1.0)),
				vec_add(br_corner_pos, Vector3::new(1.0, 1.0, 0.0)),
			),
			BlockFace::YNeg => (
				vec_add(tl_corner_pos, Vector3::new(0.0, 0.0, 0.0)),
				vec_add(br_corner_pos, Vector3::new(1.0, 0.0, 1.0)),
			),
			BlockFace::ZPos => (
				vec_add(tl_corner_pos, Vector3::new(1.0, 1.0, 1.0)),
				vec_add(br_corner_pos, Vector3::new(0.0, 0.0, 1.0)),
			),
			BlockFace::ZNeg => (
				vec_add(tl_corner_pos, Vector3::new(0.0, 1.0, 0.0)),
				vec_add(br_corner_pos, Vector3::new(1.0, 0.0, 0.0)),
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

		Self([
			 BlockVertex::new(tl_corner, segment.tl()),
			 BlockVertex::new(bl_corner, segment.bl()),
			 BlockVertex::new(br_corner, segment.br()),
			 BlockVertex::new(tr_corner, segment.tr()),
		])
	}

	// TODO: this is probably more complicated than it needs to be
	pub fn from_cube_corners(face: BlockFace, segment: TextureSegment, neg_corner_block: BlockPos, pos_corner_block: BlockPos) -> Self {
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

		Self::from_corners(face, segment, tl_corner, br_corner)
	}

	// returns the indicies of the block model to be used for the index buffer
	pub const fn indicies() -> &'static [u32] {
		&[0, 2, 1, 2, 0, 3]
	}
}

// a rectangular cutout of the texture map
#[derive(Debug, Clone, Copy)]
pub struct TextureSegment {
	top_left: TexPos,
	bottom_right: TexPos,
}

impl TextureSegment {
	const fn new(top_left: TexPos, bottom_right: TexPos) -> Self {
		let tlx = vec2_getx(top_left) + TEX_OFFSET;
		let tly = vec2_gety(top_left) + TEX_OFFSET;

		let brx = vec2_getx(bottom_right) - TEX_OFFSET;
		let bry = vec2_gety(bottom_right) - TEX_OFFSET;

		Self {
			top_left: scale_tex_pos(TexPos::new(tlx, tly)),
			bottom_right: scale_tex_pos(TexPos::new(brx, bry)),
		}
	}

	const fn from_tl(top_left: TexPos) -> Self {
		let bottom_right = TexPos::new(vec2_getx(top_left) + 1.0, vec2_gety(top_left) + 1.0);
		Self::new(top_left, bottom_right)
	}

	const fn tl(&self) -> TexPos {
		self.top_left
	}

	const fn tr(&self) -> TexPos {
		TexPos::new(vec2_getx(self.top_left), vec2_gety(self.bottom_right))
	}

	const fn bl(&self) -> TexPos {
		TexPos::new(vec2_getx(self.bottom_right), vec2_gety(self.top_left))
	}

	const fn br(&self) -> TexPos {
		self.bottom_right
	}

	// counter clockwise rotation
	const fn rotated_90(&self) -> TextureSegment {
		TextureSegment {
			top_left: self.tr(),
			bottom_right: self.bl(),
		}
	}

	// counter clockwise rotation
	const fn rotated_180(&self) -> TextureSegment {
		TextureSegment {
			top_left: self.br(),
			bottom_right: self.tl(),
		}
	}

	// counter clockwise rotation
	const fn rotated_270(&self) -> TextureSegment {
		TextureSegment {
			top_left: self.bl(),
			bottom_right: self.tr(),
		}
	}
}

// which side of the block face the top of the texture is facing when looking straight at the face
// when looking at the x or z faces, the positive y axis is up
// when looking at the y faces, the positive x axis is the right side of the face
#[derive(Debug, Clone, Copy)]
enum TextureFace {
	Up(TextureSegment),
	Down(TextureSegment),
	Left(TextureSegment),
	Right(TextureSegment),
}

impl TextureFace {
	const fn as_rotated_segment(&self) -> TextureSegment {
		match self {
			Self::Up(segment) => *segment,
			Self::Down(segment) => segment.rotated_180(),
			Self::Left(segment) => segment.rotated_90(),
			Self::Right(segment) => segment.rotated_270(),
		}
	}
}

// for now, this only supports perfect cube blocks, in future it will support more types
#[derive(Debug, Clone, Copy)]
pub struct BlockModel {
	pub xpos_texture: TextureSegment,
	pub xneg_texture: TextureSegment,
	pub ypos_texture: TextureSegment,
	pub yneg_texture: TextureSegment,
	pub zpos_texture: TextureSegment,
	pub zneg_texture: TextureSegment,
}

impl BlockModel {
	const fn from_texture(texture: TextureFace) -> Self {
		Self {
			xpos_texture: texture.as_rotated_segment(),
			xneg_texture: texture.as_rotated_segment(),
			ypos_texture: texture.as_rotated_segment(),
			yneg_texture: texture.as_rotated_segment(),
			zpos_texture: texture.as_rotated_segment(),
			zneg_texture: texture.as_rotated_segment(),
		}
	}

	const fn from_texture_faces(
		xpos_face: TextureFace,
		xneg_face: TextureFace,
		ypos_face: TextureFace,
		yneg_face: TextureFace,
		zpos_face: TextureFace,
		zneg_face: TextureFace,
	) -> Self {
		Self {
			xpos_texture: xpos_face.as_rotated_segment(),
			xneg_texture: xneg_face.as_rotated_segment(),
			ypos_texture: ypos_face.as_rotated_segment(),
			yneg_texture: yneg_face.as_rotated_segment(),
			zpos_texture: zpos_face.as_rotated_segment(),
			zneg_texture: zneg_face.as_rotated_segment(),
		}
	}

	pub fn get_face(&self, face: BlockFace) -> TextureSegment {
		match face {
			BlockFace::XPos => self.xpos_texture,
			BlockFace::XNeg => self.xneg_texture,
			BlockFace::YPos => self.ypos_texture,
			BlockFace::YNeg => self.yneg_texture,
			BlockFace::ZPos => self.zpos_texture,
			BlockFace::ZNeg => self.zneg_texture,
		}
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
	fn model(&self) -> &'static BlockModel;
	fn is_translucent(&self) -> bool;

	fn is_air(&self) -> bool {
		self.block_type() == BlockType::Air
	}
}

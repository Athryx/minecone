use std::ops::Deref;

use nalgebra::{Vector2, Vector3};

pub use crate::render::model::Model;
use crate::util::{vec2_getx, vec2_gety};

mod stone;
pub use stone::*;

pub type BlockPos = Vector3<f64>;
pub type TexPos = Vector2<f64>;

// the width and height of the texture map in number of blocks
const TEX_MAP_BLOCK_WIDTH: f64 = 1.0;
const TEX_MAP_BLOCK_HEIGHT: f64 = 1.0;

// scales the input tex_pos where 1 unit is 1 block to be in the correct coordinates for the gpu
const fn scale_tex_pos(tex_pos: TexPos) -> TexPos {
	TexPos::new(vec2_getx(tex_pos) / TEX_MAP_BLOCK_WIDTH, vec2_gety(tex_pos) / TEX_MAP_BLOCK_HEIGHT)
}

#[derive(Debug, Clone, Copy)]
pub struct BlockVertex {
	position: BlockPos,
	tex_coord: TexPos,
}

impl BlockVertex {
	pub const fn new(position: BlockPos, tex_coord: TexPos) -> Self {
		Self {
			position,
			tex_coord,
		}
	}
}

// the front of the face is the side from which the vertexes are going in a clockwise direction
// all the BlockVertexes must also be coplanar
#[derive(Debug, Clone, Copy)]
pub struct BlockFace([BlockVertex; 4]);

impl BlockFace {
	const fn new_xpos(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(1.0, 1.0, 0.0), segment.tl()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 0.0), segment.bl()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 1.0), segment.br()),
			BlockVertex::new(BlockPos::new(1.0, 1.0, 1.0), segment.tr()),
		])
	}

	const fn new_xneg(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(0.0, 1.0, 1.0), segment.tl()),
			BlockVertex::new(BlockPos::new(0.0, 0.0, 1.0), segment.bl()),
			BlockVertex::new(BlockPos::new(0.0, 0.0, 0.0), segment.br()),
			BlockVertex::new(BlockPos::new(0.0, 1.0, 0.0), segment.tr()),
		])
	}

	const fn new_ypos(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(0.0, 1.0, 1.0), segment.tl()),
			BlockVertex::new(BlockPos::new(0.0, 1.0, 0.0), segment.bl()),
			BlockVertex::new(BlockPos::new(1.0, 1.0, 0.0), segment.br()),
			BlockVertex::new(BlockPos::new(1.0, 1.0, 1.0), segment.tr()),
		])
	}

	const fn new_yneg(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(0.0, 0.0, 0.0), segment.tl()),
			BlockVertex::new(BlockPos::new(0.0, 0.0, 1.0), segment.bl()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 1.0), segment.br()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 0.0), segment.tr()),
		])
	}

	const fn new_zpos(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(1.0, 1.0, 1.0), segment.tl()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 1.0), segment.bl()),
			BlockVertex::new(BlockPos::new(0.0, 0.0, 1.0), segment.br()),
			BlockVertex::new(BlockPos::new(0.0, 1.0, 1.0), segment.tr()),
		])
	}

	const fn new_zneg(segment: TextureSegment) -> Self {
		Self([
			BlockVertex::new(BlockPos::new(0.0, 1.0, 0.0), segment.tl()),
			BlockVertex::new(BlockPos::new(0.0, 0.0, 0.0), segment.bl()),
			BlockVertex::new(BlockPos::new(1.0, 0.0, 0.0), segment.br()),
			BlockVertex::new(BlockPos::new(1.0, 1.0, 0.0), segment.tr()),
		])
	}
}

// a rectangular cutout of the texture map
#[derive(Debug, Clone, Copy)]
struct TextureSegment {
	top_left: TexPos,
	bottom_right: TexPos,
}

impl TextureSegment {
	const fn new(top_left: TexPos, bottom_right: TexPos) -> Self {
		Self {
			top_left: scale_tex_pos(top_left),
			bottom_right: scale_tex_pos(bottom_right),
		}
	}

	const fn from_tl(top_left: TexPos) -> Self {
		Self {
			top_left: scale_tex_pos(top_left),
			bottom_right: scale_tex_pos(TexPos::new(vec2_getx(top_left) + 1.0, vec2_gety(top_left) + 1.0)),
		}
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
	pub xpos: BlockFace,
	pub xneg: BlockFace,
	pub ypos: BlockFace,
	pub yneg: BlockFace,
	pub zpos: BlockFace,
	pub zneg: BlockFace,
}

impl BlockModel {
	const fn from_texture(texture: TextureFace) -> Self {
		Self::from_texture_faces(
			texture,
			texture,
			texture,
			texture,
			texture,
			texture,
		)
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
			xpos: BlockFace::new_xpos(xpos_face.as_rotated_segment()),
			xneg: BlockFace::new_xneg(xneg_face.as_rotated_segment()),
			ypos: BlockFace::new_ypos(ypos_face.as_rotated_segment()),
			yneg: BlockFace::new_yneg(yneg_face.as_rotated_segment()),
			zpos: BlockFace::new_zpos(zpos_face.as_rotated_segment()),
			zneg: BlockFace::new_zneg(zneg_face.as_rotated_segment()),
		}
	}
}

pub enum BlockType {
	Stone,
}

pub trait Block {
	fn name(&self) -> &str;
	fn block_type(&self) -> BlockType;
	fn model(&self) -> &'static BlockModel;
}

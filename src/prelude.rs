use std::ops::Deref;

use nalgebra::{Vector2, Vector3, Vector4};
use crate::game::CHUNK_SIZE;

pub type ChunkPos = Vector3<i64>;
pub type BlockPos = Vector3<i64>;
pub type Position = Vector3<f64>;

// these traites return the vector with their x, y, or z component only, and the rest zeroes
pub trait Xonly {
	fn xonly(&self) -> Self;
}

pub trait Yonly {
	fn yonly(&self) -> Self;
}

pub trait Zonly {
	fn zonly(&self) -> Self;
}

pub trait XYonly {
	fn xyonly(&self) -> Self;
}

pub trait YZonly {
	fn yzonly(&self) -> Self;
}

pub trait XZonly {
	fn xzonly(&self) -> Self;
}

impl<T: Copy + Default> Xonly for Vector2<T> {
	fn xonly(&self) -> Self {
		Self::new(self[0], T::default())
	}
}

impl<T: Copy + Default> Yonly for Vector2<T> {
	fn yonly(&self) -> Self {
		Self::new(T::default(), self[1])
	}
}


impl<T: Copy + Default> Xonly for Vector3<T> {
	fn xonly(&self) -> Self {
		Self::new(self[0], T::default(), T::default())
	}
}

impl<T: Copy + Default> Yonly for Vector3<T> {
	fn yonly(&self) -> Self {
		Self::new(T::default(), self[1], T::default())
	}
}

impl<T: Copy + Default> Zonly for Vector3<T> {
	fn zonly(&self) -> Self {
		Self::new(T::default(), T::default(), self[2])
	}
}

impl<T: Copy + Default> XYonly for Vector3<T> {
	fn xyonly(&self) -> Self {
		Self::new(self[0], self[1], T::default())
	}
}

impl<T: Copy + Default> YZonly for Vector3<T> {
	fn yzonly(&self) -> Self {
		Self::new(T::default(), self[1], self[2])
	}
}

impl<T: Copy + Default> XZonly for Vector3<T> {
	fn xzonly(&self) -> Self {
		Self::new(self[0], T::default(), self[2])
	}
}


impl<T: Copy + Default> Xonly for Vector4<T> {
	fn xonly(&self) -> Self {
		Self::new(self[0], T::default(), T::default(), T::default())
	}
}

impl<T: Copy + Default> Yonly for Vector4<T> {
	fn yonly(&self) -> Self {
		Self::new(T::default(), self[1], T::default(), T::default())
	}
}

impl<T: Copy + Default> Zonly for Vector4<T> {
	fn zonly(&self) -> Self {
		Self::new(T::default(), T::default(), self[2], T::default())
	}
}

impl<T: Copy + Default> XYonly for Vector4<T> {
	fn xyonly(&self) -> Self {
		Self::new(self[0], self[1], T::default(), T::default())
	}
}

impl<T: Copy + Default> YZonly for Vector4<T> {
	fn yzonly(&self) -> Self {
		Self::new(T::default(), self[1], self[2], T::default())
	}
}

impl<T: Copy + Default> XZonly for Vector4<T> {
	fn xzonly(&self) -> Self {
		Self::new(self[0], T::default(), self[2], T::default())
	}
}


pub trait ChunkPosExt {
	// returns the BlockPos of the bottom left corner
	fn into_block_pos(&self) -> BlockPos;
}

impl ChunkPosExt for ChunkPos {
	fn into_block_pos(&self) -> BlockPos {
		let x = self.x * CHUNK_SIZE as i64;
		let y = self.y * CHUNK_SIZE as i64;
		let z = self.z * CHUNK_SIZE as i64;
		BlockPos::new(x, y, z)
	}
}

pub trait BlockPosExt {
	fn is_chunk_local(&self) -> bool;
	fn make_chunk_local(&self) -> BlockPos;
	fn into_chunk_pos(&self) -> ChunkPos;
	// combines into_chunk_pos and make_chunk_local into 1 call
	fn into_chunk_block_pos(&self) -> (ChunkPos, BlockPos);
	fn magnitude(&self) -> f64;
}

impl BlockPosExt for BlockPos {
	fn is_chunk_local(&self) -> bool {
		self.x >= 0
			&& self.x < CHUNK_SIZE as i64
			&& self.y >= 0
			&& self.y < CHUNK_SIZE as i64
			&& self.z >= 0
			&& self.z < CHUNK_SIZE as i64
	}

	fn make_chunk_local(&self) -> BlockPos {
		let x = if self.x >= 0 {
			self.x % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + ((self.x + 1) % CHUNK_SIZE as i64) - 1
		};

		let y = if self.y >= 0 {
			self.y % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + ((self.y + 1) % CHUNK_SIZE as i64) - 1
		};

		let z = if self.z >= 0 {
			self.z % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + ((self.z + 1) % CHUNK_SIZE as i64) - 1
		};

		BlockPos::new(x, y, z)
	}

	fn into_chunk_pos(&self) -> ChunkPos {
		let x = if self.x > 0 {
			self.x / CHUNK_SIZE as i64
		} else {
			(self.x - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		let y = if self.y > 0 {
			self.y / CHUNK_SIZE as i64
		} else {
			(self.y - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		let z = if self.z > 0 {
			self.z / CHUNK_SIZE as i64
		} else {
			(self.z - (CHUNK_SIZE  as i64 - 1)) / CHUNK_SIZE as i64
		};

		ChunkPos::new(x, y, z)
	}

	fn into_chunk_block_pos(&self) -> (ChunkPos, BlockPos) {
		(self.into_chunk_pos(), self.make_chunk_local())
	}

	fn magnitude(&self) -> f64 {
		let x = self.x as f64;
		let y = self.y as f64;
		let z = self.z as f64;
		(x * x + y * y + z * z).sqrt()
	}
}

pub trait PositionExt {
	fn into_block_pos(&self) -> BlockPos;
	fn into_chunk_pos(&self) -> ChunkPos;
}

impl PositionExt for Position {
	fn into_block_pos(&self) -> BlockPos {
		BlockPos::new(self[0].floor() as i64, self[1].floor() as i64, self[2].floor() as i64)
	}

	fn into_chunk_pos(&self) -> ChunkPos{
		self.into_block_pos().into_chunk_pos()
	}
}

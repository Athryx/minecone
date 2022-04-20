use nalgebra::Vector3;
use crate::game::CHUNK_SIZE;

pub type ChunkPos = Vector3<i64>;
pub type BlockPos = Vector3<i64>;
pub type Position = Vector3<f64>;

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
			CHUNK_SIZE as i64 + (self.x % CHUNK_SIZE as i64)
		};

		let y = if self.y >= 0 {
			self.y % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + (self.y % CHUNK_SIZE as i64)
		};

		let z = if self.z >= 0 {
			self.z % CHUNK_SIZE as i64
		} else {
			CHUNK_SIZE as i64 + (self.z % CHUNK_SIZE as i64)
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
}

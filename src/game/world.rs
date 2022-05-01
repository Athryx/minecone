use std::{
	fs::{File, OpenOptions},
	path::Path,
	sync::{Arc, Weak},
};

use rustc_hash::FxHashMap;
use dashmap::DashMap;
use nalgebra::Vector3;
use anyhow::Result;
use rayon::prelude::*;
use parking_lot::RwLock;

use super::{
	chunk::{Chunk, LoadedChunk, ChunkData, VisitedBlockMap},
	entity::Entity,
	block::{BlockFaceMesh, BlockFace, Block, Stone},
	worldgen::WorldGenerator,
	player::{Player, PlayerId}, CHUNK_SIZE,
	parallel::{Task, run_task, pull_completed_task},
};
use crate::prelude::*;

// max size of world in chunks
// 16,384 meters in each x and y direction
// 2,048 meters in z direction
pub const WORLD_MAX_SIZE: Vector3<u64> = Vector3::new(512, 64, 512);

pub struct World {
	self_weak: Weak<Self>,
	players: RwLock<FxHashMap<PlayerId, Player>>,
	entities: RwLock<Vec<Box<dyn Entity>>>,
	pub chunks: FxDashMap<ChunkPos, LoadedChunk>,
	cached_chunks: RwLock<FxHashMap<ChunkPos, ChunkData>>,
	world_generator: WorldGenerator,
	// backing file of the world
	file: File,
}

impl World {
	pub fn load_from_file<T: AsRef<Path>>(file_name: T) -> Result<Arc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(file_name)?;

		Ok(Arc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RwLock::new(FxHashMap::default()),
			entities: RwLock::new(Vec::new()),
			chunks: FxDashMap::default(),
			cached_chunks: RwLock::new(FxHashMap::default()),
			world_generator: WorldGenerator::new(0),
			file,
		}))
	}

	// TEMP
	pub fn new_test() -> Result<Arc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open("test-world")?;

		let out = Arc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RwLock::new(FxHashMap::default()),
			entities: RwLock::new(Vec::new()),
			chunks: FxDashMap::default(),
			cached_chunks: RwLock::new(FxHashMap::default()),
			world_generator: WorldGenerator::new(0),
			file,
		});

		Ok(out)
	}

	// TODO: load and unload queue mesh updates
	// loads all chunks between min_chunk and max_chunk not including max_chunk,
	// or incraments the load count if they are already loaded
	pub fn load_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					let chunk = self.chunks.entry(position)
						.or_insert_with(|| self.world_generator
							.generate_chunk(self.self_weak.upgrade().unwrap(), position));

					// when first inserting load count starts at 0
					chunk.inc_load_count();
				}
			}
		}
	}

	// decraments the load counter of all chunks between min and max chunk, not including max
	// and unloads them if the count reaches 0
	pub fn unload_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					if let Some(loaded_chunk) = self.chunks.get(&position) {
						if loaded_chunk.dec_load_count() == 0 {
							drop(loaded_chunk);
							self.chunks.remove(&position);
						}
					}
				}
			}
		}
	}

	// performs mesh updates on the passed in block as well as all adjacent blocks
	// FIXME: this doesn't update everything it needs to with ambient occlusion on chunk boundaries
	pub fn mesh_update_adjacent(&self, block: BlockPos) {
		let block_chunk_local = block.as_chunk_local();
		let mut visit_map = VisitedBlockMap::new();

		if let Some(chunk) = self.chunks.get(&block.as_chunk_pos()) {
			chunk.chunk.mesh_update_inner(BlockFace::XPos, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::XNeg, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YPos, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YNeg, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZPos, block_chunk_local.z as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZNeg, block_chunk_local.z as usize, &mut visit_map);
		}

		for face in BlockFace::iter() {
			// subtract to get opposite as normal offest
			let offset_block = block - face.block_pos_offset();
			if let Some(chunk) = self.chunks.get(&offset_block.as_chunk_pos()) {
				chunk.chunk.mesh_update_inner(
					face,
					offset_block.as_chunk_local().get_face_component(face) as usize,
					&mut visit_map
				);
			}
		}
	}

	pub fn chunk_mesh_update(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let chunk_pos = ChunkPos::new(x, y, z);
					run_task(Task::ChunkMesh(chunk_pos));
				}
			}
		}
	}

	// performs a mesh update on 1 side of the chunk for all specified chunks
	pub fn chunk_mesh_update_face(&self, face: BlockFace, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		let mut visit_map = VisitedBlockMap::new();

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let chunk_pos = BlockPos::new(x, y, z);
					if let Some(chunk) = self.chunks.get(&chunk_pos) {
						let index = if face.is_positive_face() {
							CHUNK_SIZE - 1
						} else {
							0
						};

						chunk.chunk.mesh_update_inner(face, index, &mut visit_map);
					}
				}
			}
		}
	}

	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&dyn Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(&*self.chunks.get(&chunk_position)?
			.chunk.get_block(block.as_chunk_local())))
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(&mut *self.chunks.get(&chunk_position)?
			.chunk.get_block_mut(block.as_chunk_local())))
	}

	// sets the block at BlockPos, returns bool on success
	pub fn set_block(&self, block_pos: BlockPos, block: Box<dyn Block>) -> bool {
		let (chunk_pos, block_pos) = block_pos.as_chunk_block_pos();

		if let Some(chunk) = self.chunks.get(&chunk_pos) {
			chunk.chunk.set_block(block_pos, block);
			true
		} else {
			false
		}
	}

	// casts a ray starting at ray_start up to a length of max_length
	// if a block other than air is found, the coordinates are returned, otherwise None is returned
	// if the ray ever intersects with an empty chunk, None is returned
	// FIXME: ugly
	pub fn block_raycast(&self, ray_start: Position, ray: Vector3<f64>, max_length: f64) -> Option<BlockPos> {
		let ray = ray.normalize();
		let block_start_pos = ray_start.into_block_pos();
		let mut block_pos = block_start_pos;

		let direction_x = if ray.x > 0.0 { 1 } else if ray.x < 0.0 { -1 } else { 0 };
		let direction_y = if ray.y > 0.0 { 1 } else if ray.y < 0.0 { -1 } else { 0 };
		let direction_z = if ray.z > 0.0 { 1 } else if ray.z < 0.0 { -1 } else { 0 };

		let intercept_time_interval_x = if ray.x != 0.0 { (1.0 / ray.x).abs() } else { f64::INFINITY };
		let intercept_time_interval_y = if ray.y != 0.0 { (1.0 / ray.y).abs() } else { f64::INFINITY };
		let intercept_time_interval_z = if ray.z != 0.0 { (1.0 / ray.z).abs() } else { f64::INFINITY };

		let ray_offset_x = if ray_start.x > 0.0 { ray_start.x % 1.0 } else { 1.0 + (ray_start.x % 1.0) };
		let ray_offset_y = if ray_start.y > 0.0 { ray_start.y % 1.0 } else { 1.0 + (ray_start.y % 1.0) };
		let ray_offset_z = if ray_start.z > 0.0 { ray_start.z % 1.0 } else { 1.0 + (ray_start.z % 1.0) };

		let mut next_intercept_time_x = if ray.x > 0.0 { (1.0 - ray_offset_x) / ray.x } else if ray.x < 0.0 { ray_offset_x / -ray.x } else { f64::INFINITY };
		let mut next_intercept_time_y = if ray.y > 0.0 { (1.0 - ray_offset_y) / ray.y } else if ray.y < 0.0 { ray_offset_y / -ray.y } else { f64::INFINITY };
		let mut next_intercept_time_z = if ray.z > 0.0 { (1.0 - ray_offset_z) / ray.z } else if ray.z < 0.0 { ray_offset_z / -ray.z } else { f64::INFINITY };

		loop {
			if next_intercept_time_x < next_intercept_time_y && next_intercept_time_x < next_intercept_time_z {
				block_pos.x += direction_x;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_x += intercept_time_interval_x;
			} else if next_intercept_time_y < next_intercept_time_z {
				block_pos.y += direction_y;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_y += intercept_time_interval_y;
			} else {
				block_pos.z += direction_z;
				if (block_pos - block_start_pos).magnitude() > max_length {
					return None;
				}

				if !self.with_block(block_pos, |b| b.is_air())? {
					return Some(block_pos);
				}

				next_intercept_time_z += intercept_time_interval_z;
			}
		}
	}

	// called by the client to force the world to recieve task completion notices
	// returns true if the mesh should be updated by the client
	pub fn poll_completed_tasks(&self) -> bool {
		let mut update_mesh = false;
		while let Some(task) = pull_completed_task() {
			match task {
				Task::ChunkMesh(_) => update_mesh = true,
			}
		}
		update_mesh
	}
}

impl World {
	pub fn connect(&self) -> PlayerId {
		let player = Player::new();

		let min_load_chunk = player.chunk_position() - player.render_distance();
		let max_load_chunk = player.chunk_position() + player.render_distance();
		self.load_chunks(min_load_chunk, max_load_chunk);
		self.chunk_mesh_update(min_load_chunk, max_load_chunk);

		let id = player.id();
		self.players.write().insert(id, player);
		id
	}

	// TODO: allow changing from more than 1 chunk at at a time
	// TODO: when going along diaganols, sometimes chunks are loaded and immediately unloaded
	// TEMP: returns true if mesh has changed
	// FIXME: ugly
	pub fn set_player_position(&self, player_id: PlayerId, position: Position) -> Option<bool> {
		let mut players = self.players.write();
		let player = players.get_mut(&player_id)?;

		let chunk_position = position.into_chunk_pos();

		let render_zone_corner = player.chunk_position() - player.render_distance();
		let render_zone_length = 2 * player.render_distance();

		if chunk_position.x != player.chunk_position().x {
			let xaxis = ChunkPos::new(1, 0, 0);

			let pos_min_chunk = render_zone_corner + render_zone_length.xonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + xaxis;

			let neg_min_chunk = render_zone_corner - xaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.yzonly();

			if chunk_position.x == player.chunk_position().x + 1 {
				let neg_min_chunk = neg_min_chunk + xaxis;
				let neg_max_chunk = neg_max_chunk + xaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update_face(BlockFace::XPos, neg_min_chunk - BlockPos::new(1, 0, 0), neg_max_chunk - BlockPos::new(1, 0, 0));

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.x == player.chunk_position().x - 1 {
				let pos_min_chunk = pos_min_chunk - xaxis;
				let pos_max_chunk = pos_max_chunk - xaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update_face(BlockFace::XNeg, neg_min_chunk + BlockPos::new(1, 0, 0), neg_max_chunk + BlockPos::new(1, 0, 0));

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		if chunk_position.y != player.chunk_position().y {
			let yaxis = ChunkPos::new(0, 1, 0);

			let pos_min_chunk = render_zone_corner + render_zone_length.yonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + yaxis;

			let neg_min_chunk = render_zone_corner - yaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.xzonly();

			if chunk_position.y == player.chunk_position().y + 1 {
				let neg_min_chunk = neg_min_chunk + yaxis;
				let neg_max_chunk = neg_max_chunk + yaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update_face(BlockFace::YPos, neg_min_chunk - BlockPos::new(0, 1, 0), neg_max_chunk - BlockPos::new(0, 1, 0));

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.y == player.chunk_position().y - 1 {
				let pos_min_chunk = pos_min_chunk - yaxis;
				let pos_max_chunk = pos_max_chunk - yaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update_face(BlockFace::YNeg, neg_min_chunk + BlockPos::new(0, 1, 0), neg_max_chunk + BlockPos::new(0, 1, 0));

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		if chunk_position.z != player.chunk_position().z {
			let zaxis = ChunkPos::new(0, 0, 1);

			let pos_min_chunk = render_zone_corner + render_zone_length.zonly();
			let pos_max_chunk = render_zone_corner + render_zone_length + zaxis;

			let neg_min_chunk = render_zone_corner - zaxis;
			let neg_max_chunk = render_zone_corner + render_zone_length.xyonly();

			if chunk_position.z == player.chunk_position().z + 1 {
				let neg_min_chunk = neg_min_chunk + zaxis;
				let neg_max_chunk = neg_max_chunk + zaxis;

				self.unload_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update_face(BlockFace::ZPos, neg_min_chunk - BlockPos::new(0, 0, 1), neg_max_chunk - BlockPos::new(0, 0, 1));

				self.load_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
			} else if chunk_position.z == player.chunk_position().z - 1 {
				let pos_min_chunk = pos_min_chunk - zaxis;
				let pos_max_chunk = pos_max_chunk - zaxis;

				self.unload_chunks(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update(pos_min_chunk, pos_max_chunk);
				self.chunk_mesh_update_face(BlockFace::ZNeg, neg_min_chunk + BlockPos::new(0, 0, 1), neg_max_chunk + BlockPos::new(0, 0, 1));

				self.load_chunks(neg_min_chunk, neg_max_chunk);
				self.chunk_mesh_update(neg_min_chunk, neg_max_chunk);
			} else {
				todo!("moved to far for current player moving code");
			}
		}

		let out = chunk_position != player.chunk_position();

		player.position = position;
		Some(out)
	}

	pub fn world_mesh(&self) -> Vec<BlockFaceMesh> {
		self.chunks.iter()
			.filter_map(|item| item.value().chunk.get_chunk_mesh())
			.flatten()
			.collect::<Vec<_>>()
	}
}

#[cfg(test)]
mod tests {
	extern crate test;

	use test::Bencher;
	use super::*;

	#[bench]
	fn mesh_generation_benchmark(b: &mut Bencher) {
		b.iter(|| {
			let world = World::new_test().unwrap();
			world.connect();
		})
	}
}

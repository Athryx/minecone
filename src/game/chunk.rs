use std::rc::Rc;
use std::cell::RefCell;

use array_init::array_init;

use super::block::{Block, BlockFaceMesh, BlockFace, OcclusionCorners};
// TEMP
use super::block::{Air, Stone};
use super::entity::Entity;
use super::world::World;
use crate::prelude::*;

use crate::array3d_init;

pub const CHUNK_SIZE: usize = 32;

// says all blocks that have been visited for the greedy meshing algorithm in a given layer
pub struct VisitedBlockMap {
	visited_blocks: Box<[[bool; CHUNK_SIZE]; CHUNK_SIZE]>,
	vertex_occlusion: Box<[[u8; CHUNK_SIZE + 1]; CHUNK_SIZE + 1]>,
	face: BlockFace,
	coord3: i64,
}

impl VisitedBlockMap {
	// the face and coord3 are used to return the correct number in the unused coordinate of the 3rd vector
	// face and coord3 should be set according to the current slice we are iterating over
	pub fn new() -> Self {
		VisitedBlockMap {
			visited_blocks: Box::new([[false; CHUNK_SIZE]; CHUNK_SIZE]),
			vertex_occlusion: Box::new([[0; CHUNK_SIZE + 1]; CHUNK_SIZE + 1]),
			face: BlockFace::XPos,
			coord3: 0,
		}
	}

	fn get_index(&self, position: BlockPos) -> (usize, usize) {
		let (x, y) = match self.face {
			BlockFace::XPos | BlockFace::XNeg => (position.y, position.z),
			BlockFace::YPos | BlockFace::YNeg => (position.x, position.z),
			BlockFace::ZPos | BlockFace::ZNeg => (position.x, position.y),
		};
		(x.try_into().unwrap(), y.try_into().unwrap())
	}
	
	fn get_block_pos(&self, x: i64, y: i64) -> BlockPos {
		match self.face {
			BlockFace::XPos | BlockFace::XNeg => BlockPos::new(self.coord3, x, y),
			BlockFace::YPos | BlockFace::YNeg => BlockPos::new(x, self.coord3, y),
			BlockFace::ZPos | BlockFace::ZNeg => BlockPos::new(x, y, self.coord3),
		}
	}

	fn get_block_pos_offset(&self, block: BlockPos, x_offset: i64, y_offset: i64) -> BlockPos {
		match self.face {
			BlockFace::XPos | BlockFace::XNeg => BlockPos::new(self.coord3, block.y + x_offset, block.z + y_offset),
			BlockFace::YPos | BlockFace::YNeg => BlockPos::new(block.x + x_offset, self.coord3, block.z + y_offset),
			BlockFace::ZPos | BlockFace::ZNeg => BlockPos::new(block.x + x_offset, block.y + y_offset, self.coord3),
		}
	}

	fn is_visited(&self, position: BlockPos) -> bool {
		let (x, y) = self.get_index(position);
		self.visited_blocks[x][y]
	}

	fn set_visited(&mut self, position: BlockPos, visited: bool) {
		let (x, y) = self.get_index(position);
		self.visited_blocks[x][y] = visited;
	}

	fn set_occlusion_level(&mut self, x: i64, y: i64, occlusion_level: u8) {
		let x: usize = x.try_into().unwrap();
		let y: usize = y.try_into().unwrap();
		self.vertex_occlusion[x][y] = occlusion_level;
	}

	fn occlusion_level_matches(&self, block1: BlockPos, block2: BlockPos) -> bool {
		let (x1, y1) = self.get_index(block1);
		let (x2, y2) = self.get_index(block2);
		self.vertex_occlusion[x1][y1] == self.vertex_occlusion[x2][y2]
			&& self.vertex_occlusion[x1 + 1][y1] == self.vertex_occlusion[x2 + 1][y2]
			&& self.vertex_occlusion[x1][y1 + 1] == self.vertex_occlusion[x2][y2 + 1]
			&& self.vertex_occlusion[x1 + 1][y1 + 1] == self.vertex_occlusion[x2 + 1][y2 + 1]
	}

	fn get_occlusion_data(&self, block: BlockPos) -> OcclusionCorners {
		let (x, y) = self.get_index(block);
		OcclusionCorners {
			tl: self.vertex_occlusion[x][y + 1],
			tr: self.vertex_occlusion[x + 1][y + 1],
			bl: self.vertex_occlusion[x][y],
			br: self.vertex_occlusion[x + 1][y],
		}
	}

	fn set_face_coord(&mut self, face: BlockFace, coord3: i64) {
		self.face = face;
		self.coord3 = coord3;
	}
}

pub struct Chunk {
	world: Rc<World>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: Position,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	// coordinates of bottom left back block in world space
	block_position: BlockPos,
	// store them on heap to avoid stack overflow
	blocks: Box<[[[Box<dyn Block>; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
	//chunk_mesh: HashMap<BlockPos, Vec<BlockFaceMesh>>,
	chunk_mesh: Box<[[Vec<BlockFaceMesh>; CHUNK_SIZE]; 6]>,
}

impl Chunk {
	// TEMP
	pub fn new_test(world: Rc<World>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;

		let mut blocks = Box::new(array3d_init!(Stone::new()));
		blocks[5][16][24] = Air::new();
		blocks[0][0][0] = Air::new();
		blocks[13][19][0] = Air::new();

		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			block_position: position * CHUNK_SIZE as i64,
			blocks,
			chunk_mesh: Box::new(array_init(|_| array_init(|_| Vec::new()))),
		}
	}

	pub fn new_test_air(world: Rc<World>, position: ChunkPos) -> Self {
		let x = (position.x * CHUNK_SIZE as i64) as f64;
		let y = (position.y * CHUNK_SIZE as i64) as f64;
		let z = (position.z * CHUNK_SIZE as i64) as f64;
		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			block_position: position * CHUNK_SIZE as i64,
			blocks: Box::new(array3d_init!(Air::new())),
			chunk_mesh: Box::new(array_init(|_| array_init(|_| Vec::new()))),
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&dyn Block) -> T {
		if block.is_chunk_local() {
			Some(f(self.get_block(block)))
		} else {
			let chunk_position = block.as_chunk_pos() + self.chunk_position;

			Some(f(self.world
				.chunks.borrow().get(&chunk_position)?.borrow()
				.chunk.get_block(block.as_chunk_local())))
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		if block.is_chunk_local() {
			Some(f(self.get_block_mut(block)))
		} else {
			let chunk_position = block.as_chunk_pos() + self.chunk_position;

			Some(f(self.world
				.chunks.borrow().get(&chunk_position)?.borrow_mut()
				.chunk.get_block_mut(block.as_chunk_local())))
		}
	}

	#[inline]
	pub fn get_block(&self, block: BlockPos) -> &dyn Block {
		let x: usize = block.x.try_into().unwrap();
		let y: usize = block.y.try_into().unwrap();
		let z: usize = block.z.try_into().unwrap();
		&*self.blocks[x][y][z]
	}

	#[inline]
	pub fn get_block_mut(&mut self, block: BlockPos) -> &mut dyn Block {
		let x: usize = block.x.try_into().unwrap();
		let y: usize = block.y.try_into().unwrap();
		let z: usize = block.z.try_into().unwrap();
		&mut *self.blocks[x][y][z]
	}

	#[inline]
	pub fn set_block(&mut self, block_pos: BlockPos, block: Box<dyn Block>) {
		assert!(block_pos.is_chunk_local());
		self.blocks[block_pos.x as usize][block_pos.y as usize][block_pos.z as usize] = block;
	}

	// the visit map is passed in seperately to avoid having to reallocat the memory for the visit map every time	
	pub fn mesh_update_inner(&mut self, face: BlockFace, index: usize, visit_map: &mut VisitedBlockMap) {
		visit_map.set_face_coord(face, index as i64);
		self.chunk_mesh[Into::<usize>::into(face)][index].clear();

		// discard all block faces that are not visible and all faces on an air block
		for x in 0..CHUNK_SIZE as i64 {
			for y in 0..CHUNK_SIZE as i64 {
				let block_pos = visit_map.get_block_pos(x, y);

				if self.get_block(block_pos).is_air() {
					visit_map.set_visited(block_pos, true);
				} else if let Some(is_translucent) = self.with_block(block_pos + face.block_pos_offset(), |block| block.is_translucent()) {
					visit_map.set_visited(block_pos, !is_translucent);
				} else {
					// there is no adjacent chunk, don't do this mesh
					visit_map.set_visited(block_pos, true);
				}
			}
		}

		// get occlusion levels of all verticies
		for x in 0..(CHUNK_SIZE as i64 + 1) {
			for y in 0..(CHUNK_SIZE as i64 + 1) {
				let is_occluded_by = |x, y| {
					let block_pos = visit_map.get_block_pos(x, y);
					if let Some(is_translucent) = self.with_block(block_pos + face.block_pos_offset(), |block| block.is_translucent()) {
						if is_translucent {
							0
						} else {
							1
						}
					} else {
						0
					}
				};

				let tl_occludes = is_occluded_by(x - 1, y - 1);
				let tr_occludes = is_occluded_by(x, y - 1);
				let bl_occludes = is_occluded_by(x - 1, y);
				let br_occludes = is_occluded_by(x, y);

				let mut occlusion_level = tl_occludes + tr_occludes + bl_occludes + br_occludes;
				// if the vertex is in a corner formed by only 2 blocks, the occlusion level needs to be 3
				if (tl_occludes == 1 && br_occludes == 1) || (tr_occludes == 1 && bl_occludes == 1) {
					occlusion_level = 3;
				}

				visit_map.set_occlusion_level(x, y, occlusion_level);
			}
		}

		for x in 0..CHUNK_SIZE as i64 {
			for y in 0..CHUNK_SIZE as i64 {
				let block_pos = visit_map.get_block_pos(x, y);
				if visit_map.is_visited(block_pos) {
					continue;
				}

				let block = self.get_block(block_pos);
				let block_type = block.block_type();
	
				// width and height of the greedy mesh region
				let mut width = 1;
				let mut height = 0;
	
				loop {
					let current_block_pos = visit_map.get_block_pos_offset(block_pos, width, 0);
					// get_block_pos_offset could put current block out of bounds
					if !current_block_pos.is_chunk_local() {
						break;
					}
	
					if !visit_map.is_visited(current_block_pos)
						&& self.get_block(current_block_pos).block_type() == block_type
						&& visit_map.occlusion_level_matches(block_pos, current_block_pos) {
						visit_map.set_visited(current_block_pos, true);
						width += 1;
					} else {
						// don't visit a block if it is a different type, we still want to visit later
						break;
					}
				}
	
				height += 1;
	
				loop {
					// can we expand the height of the mesh
					let mut expandable = true;
	
					for w in 0..width {
						let current_block_pos = visit_map.get_block_pos_offset(block_pos, w, height);
						// get_block_pos_offset could put current block out of bounds
						if !current_block_pos.is_chunk_local() {
							// if we can't test all of the blocks, we can't expand the region
							expandable = false;
							break;
						}

						if visit_map.is_visited(current_block_pos)
							|| self.get_block(current_block_pos).block_type() != block_type
							|| !visit_map.occlusion_level_matches(block_pos, current_block_pos) {
							expandable = false;
							break;
						}
					}
	
					if !expandable {
						break;
					}
	
					for w in 0..width {
						visit_map.set_visited(visit_map.get_block_pos_offset(block_pos, w, height), true);
					}
	
					height += 1;
				}
	
				let block_face_mesh = BlockFaceMesh::from_cube_corners(
					face,
					block.texture_index(),
					block_pos + self.block_position,
					visit_map.get_block_pos_offset(block_pos, width - 1, height - 1) + self.block_position,
					visit_map.get_occlusion_data(block_pos),
				);
	
				self.chunk_mesh[Into::<usize>::into(face)][index].push(block_face_mesh);
			}
		}
	}

	// updates the mesh for the entire chunk
	pub fn chunk_mesh_update(&mut self) {
		let mut visit_map = VisitedBlockMap::new();

		for face in BlockFace::iter() {
			for i in 0..CHUNK_SIZE {
				self.mesh_update_inner(face, i, &mut visit_map);
			}
		}
	}

	pub fn get_chunk_mesh(&self) -> Vec<BlockFaceMesh> {
		self.chunk_mesh.iter()
			.flatten()
			.flatten()
			.copied()
			.collect::<Vec<_>>()
	}
}

pub struct LoadedChunk {
	pub chunk: Chunk,
	pub load_count: u64,
}

impl LoadedChunk {
	pub fn new(chunk: Chunk) -> RefCell<LoadedChunk> {
		//let chunk_mesh = chunk.generate_block_faces();
		RefCell::new(LoadedChunk {
			chunk,
			load_count: 0,
		})
	}
}

// the entire saved state of the chunk, which is all blocks and entities
// TODO: maybe save chunk mesh to load faster
pub struct ChunkData {
	chunk: Chunk,
	entities: Vec<Box<dyn Entity>>,
}

use std::lazy::SyncLazy;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam::{
	deque::{Injector, Steal},
	queue::SegQueue,
};

use crate::prelude::*;
use super::world::World;

static TASK_QUEUE: SyncLazy<Injector<Task>> = SyncLazy::new(|| Injector::new());
static COMPLETED_TASKS: SegQueue<Task> = SegQueue::new();

#[derive(Debug, Clone)]
pub enum Task {
	// generate a mesh for the given chunk
	ChunkMesh(ChunkPos),
}

pub fn init(world: Arc<World>, num_tasks: usize) {
	info!("runing with {} task processing threads", num_tasks);
	for _ in 0..num_tasks {
		let thread_world = world.clone();
		thread::spawn(move || task_runner(thread_world));
	}
}

// appends the given task to the task queue
pub fn run_task(task: Task) {
	TASK_QUEUE.push(task);
}

pub fn pull_completed_task() -> Option<Task> {
	COMPLETED_TASKS.pop()
}

// waits for a task to apear, than runs it
fn task_runner(world: Arc<World>) {
	let sleep_duration = Duration::from_millis(2);

	loop {
		match TASK_QUEUE.steal() {
			Steal::Success(task) => execute_task(&world, task),
			Steal::Empty => thread::sleep(sleep_duration),
			Steal::Retry => continue,
		}
	}
}

// executes the given task
fn execute_task(world: &World, task: Task) {
	match task {
		Task::ChunkMesh(chunk) => {
			world.chunks.get(&chunk).map(|chunk| chunk.value().chunk.chunk_mesh_update());
			COMPLETED_TASKS.push(task);
		},
	};
}

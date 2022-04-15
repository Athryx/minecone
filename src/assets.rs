use std::path::{PathBuf, Path};
use std::fs;
use std::lazy::SyncLazy;

use anyhow::Result;
use tobj::{Model, Material};

static LOADER: SyncLazy<AssetLoader> = SyncLazy::new(|| AssetLoader::from_path(PathBuf::from("res/")));

pub fn loader() -> &'static AssetLoader {
	&LOADER
}

// this is realy basic for now, may be improved in future
pub struct AssetLoader {
	resource_folder: PathBuf,
}

impl AssetLoader {
	fn from_path(resource_folder: PathBuf) -> Self {
		Self {
			resource_folder,
		}
	}

	fn path_of<T: AsRef<Path>>(&self, resource: T) -> PathBuf {
		let mut path = self.resource_folder.clone();
		path.push(resource);
		path
	}

	pub fn load_bytes<T: AsRef<Path>>(&self, file: T) -> Result<Vec<u8>> {
		Ok(fs::read(&self.path_of(file))?)
	}

	pub fn load_obj<T: AsRef<Path>>(&self, file: T) -> Result<(Vec<Model>, Vec<Material>)> {
		let (obj_meshes, obj_materials) = tobj::load_obj(&self.path_of(file), &tobj::GPU_LOAD_OPTIONS)?;
		let obj_materials = obj_materials?;
		Ok((obj_meshes, obj_materials))
	}
}

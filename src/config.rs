use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config<'a> {
    root_path: &'a Path,
    home_dir: PathBuf,
}

impl<'a> Config<'a> {
    pub fn new(root_path: &'a Path, home_dir: PathBuf) -> Self {
        Self {
            root_path: root_path,
            home_dir,
        }
    }

    pub fn root_path(&self) -> &'a Path {
        self.root_path
    }

    pub fn home_dir(&'a self) -> &'a Path {
        self.home_dir.as_ref()
    }
}

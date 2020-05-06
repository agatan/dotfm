use std::path::{Path, PathBuf};
use std::env;

use log::debug;
use anyhow::Error;


pub struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    pub fn new<P: AsRef<Path>>(in_dir: P) -> Result<Self, Error> {
        let original = env::current_dir()?;
        debug!(
            "move from {} to {}",
            original.to_string_lossy(),
            in_dir.as_ref().to_string_lossy()
        );
        env::set_current_dir(in_dir.as_ref())?;
        Ok(Self { original })
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        debug!("back to {}", self.original.to_string_lossy());
        env::set_current_dir(self.original.as_path()).expect("failed to reset current directory")
    }
}

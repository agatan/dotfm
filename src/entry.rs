use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::Context as _;

pub struct Entry<'a> {
    root: &'a Path,
    relative_path: PathBuf,
    target_absolute_path: PathBuf,
}

impl<'a> Entry<'a> {
    pub fn new(root: &'a Path, relative_path: PathBuf, target_absolute_path: PathBuf) -> Self {
        Entry {
            root,
            relative_path,
            target_absolute_path,
        }
    }

    pub fn absolute_path(&self) -> PathBuf {
        self.root.join(&self.relative_path)
    }

    pub fn display_relative(&self) -> DisplayRelative {
        DisplayRelative(self)
    }

    pub fn display_target<'b>(&'b self) -> impl fmt::Display + 'b {
        self.target_absolute_path.display()
    }

    pub fn is_linked(&self) -> bool {
        match std::fs::read_link(&self.target_absolute_path) {
            Err(_) => false,
            Ok(link) => link == self.absolute_path(),
        }
    }

    fn create_target_parent_dirs(&self) -> Result<(), anyhow::Error> {
        match self.target_absolute_path.parent() {
            None => Ok(()),
            Some(p) => std::fs::create_dir_all(p).context("Failed to create parent directories"),
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn link(&self) -> Result<(), anyhow::Error> {
        self.create_target_parent_dirs()?;
        std::os::unix::fs::symlink(self.absolute_path(), &self.target_absolute_path).context(
            format!(
                "Failed to link {} to {}",
                self.absolute_path().display(),
                self.target_absolute_path.display()
            ),
        )
    }

    #[cfg(target_os = "windows")]
    pub fn link(&self) -> Result<(), anyhow::Error> {
        self.create_target_parent_dirs()?;
        std::os::windows::fs::symlink_file(self.absolute_path(), &self.target_absolute_path)
            .context(format!(
                "Failed to link {} to {}",
                self.absolute_path().display(),
                self.target_absolute_path.display()
            ))
    }
}

pub struct DisplayRelative<'a>(&'a Entry<'a>);

impl<'a> fmt::Display for DisplayRelative<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.relative_path.display().fmt(f)
    }
}

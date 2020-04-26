use std::fmt;
use std::path::{Path, PathBuf};

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
}

pub struct DisplayRelative<'a>(&'a Entry<'a>);

impl<'a> fmt::Display for DisplayRelative<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.relative_path.display().fmt(f)
    }
}

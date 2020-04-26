use std::fmt;
use std::path::{Path, PathBuf};

pub struct Entry<'a> {
    root: &'a Path,
    entry_path: PathBuf,
}

impl<'a> Entry<'a> {
    pub fn new(root: &'a Path, entry_path: PathBuf) -> Self {
        Entry { root, entry_path }
    }

    pub fn relative_path(&self) -> &Path {
        self.entry_path
            .strip_prefix(self.root)
            .expect("each entry must start with the base path")
    }
}

impl<'a> fmt::Display for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.relative_path().display().fmt(f)
    }
}

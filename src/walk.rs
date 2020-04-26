use std::iter::Iterator;
use std::path::Path;

use ignore::overrides::OverrideBuilder;

use crate::entry::Entry;

pub struct Walk<'a> {
    root: &'a Path,
    ignore_walk: ignore::Walk,
}

impl<'a> Walk<'a> {
    pub fn new(root: &'a Path) -> Result<Self, anyhow::Error> {
        let overrides = OverrideBuilder::new(root)
            .add("!/.git")?
            .add("!/.dotfmignore")?
            .build()?;
        let walk = ignore::WalkBuilder::new(root)
            .hidden(false) // Do not ignore hidden files
            .add_custom_ignore_filename(".dotfmignore")
            .overrides(overrides)
            .sort_by_file_path(|p1, p2| p1.cmp(p2))
            .build();
        Ok(Walk {
            root,
            ignore_walk: walk,
        })
    }
}

impl<'a> Iterator for Walk<'a> {
    type Item = Result<Entry<'a>, anyhow::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(p) = self.ignore_walk.next() {
            let p = match p {
                Err(err) => return Some(Err(err.into())),
                Ok(p) => p,
            };
            if p.path_is_symlink() || p.path().is_dir() {
                continue;
            }
            return Some(Ok(Entry::new(self.root, p.path().to_owned())));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::prelude::*;

    #[test]
    fn test_walk_files() {
        let tempdir = assert_fs::TempDir::new().unwrap();
        tempdir.child("file-C").touch().unwrap();
        tempdir.child("file-B").touch().unwrap();
        tempdir.child("file-A").touch().unwrap();
        tempdir.child(".git").create_dir_all().unwrap();
        tempdir.child(".git").child("info").touch().unwrap();
        let walk = Walk::new(tempdir.path()).unwrap();
        assert_eq!(
            walk.map(|r| r.map(|e| format!("{}", e)))
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            vec!["file-A", "file-B", "file-C"]
        );
    }
}
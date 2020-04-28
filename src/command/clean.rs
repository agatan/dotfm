use std::path::Path;

use anyhow::Context as _;

use crate::walk::Walk;

pub fn do_clean(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let walk = Walk::new(path.as_ref())?;
    for entry in walk {
        let entry = entry?;
        entry
            .unlink()
            .context(format!("Failed to unlink {}", entry.display_target()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::prelude::*;

    #[test]
    fn test_clean() {
        let homedir = assert_fs::TempDir::new().unwrap();
        std::env::set_var("HOME", homedir.path());
        let tempdir = assert_fs::TempDir::new().unwrap();
        tempdir.child(".vimrc").touch().unwrap();

        // Link .vimrc
        crate::command::do_link(tempdir.path()).unwrap();
        assert!(homedir.child(".vimrc").path().exists());

        // Clean .vimrc
        do_clean(tempdir.path()).unwrap();
        assert!(!homedir.child(".vimrc").path().exists());

        // Ensure the original .vimrc is not deleted
        assert!(tempdir.child(".vimrc").path().exists());
    }
}

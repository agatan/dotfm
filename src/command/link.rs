use std::path::Path;

use crate::walk::Walk;

pub fn do_link(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    let walk = Walk::new(path.as_ref())?;
    for entry in walk {
        let entry = entry?;
        entry.link()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_do_link() {
        let homedir = assert_fs::TempDir::new().unwrap();
        std::env::set_var("HOME", homedir.path().as_os_str());
        let tempdir = assert_fs::TempDir::new().unwrap();
        assert_eq!(dirs::home_dir().unwrap(), homedir.path());
        tempdir.child(".vimrc").touch().unwrap();
        do_link(tempdir.path()).unwrap();
        assert_eq!(
            std::fs::read_link(homedir.child(".vimrc").path()).unwrap(),
            tempdir.child(".vimrc").path(),
        );
    }
}

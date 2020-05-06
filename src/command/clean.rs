use anyhow::Context as _;

use crate::config::Config;
use crate::walk::Walk;

pub fn do_clean(config: &Config) -> Result<(), anyhow::Error> {
    let walk = Walk::new(config.root_path(), config.home_dir())?;
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
        let tempdir = assert_fs::TempDir::new().unwrap();
        tempdir.child(".vimrc").touch().unwrap();
        let config = Config::new(tempdir.path(), homedir.path().to_owned());

        // Link .vimrc
        crate::command::do_link(&config).unwrap();
        assert!(homedir.child(".vimrc").path().exists());

        // Clean .vimrc
        do_clean(&config).unwrap();
        assert!(!homedir.child(".vimrc").path().exists());

        // Ensure the original .vimrc is not deleted
        assert!(tempdir.child(".vimrc").path().exists());
    }
}

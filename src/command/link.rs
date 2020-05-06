use crate::config::Config;
use crate::walk::Walk;

pub fn do_link(config: &Config) -> Result<(), anyhow::Error> {
    let walk = Walk::new(config.root_path(), config.home_dir())?;
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
        let tempdir = assert_fs::TempDir::new().unwrap();
        tempdir.child(".vimrc").touch().unwrap();
        let config = Config::new(tempdir.path(), homedir.path().to_owned());
        do_link(&config).unwrap();
        assert_eq!(
            std::fs::read_link(homedir.child(".vimrc").path()).unwrap(),
            tempdir.child(".vimrc").path(),
        );
    }
}

use std::ffi::OsStr;
use std::process::{Command, ExitStatus};

use crate::config::Config;
use crate::util::DirGuard;

fn run_git<I, S>(args: I) -> Result<(ExitStatus, String), anyhow::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git").args(args).output()?;
    let status = output.status;
    Ok((status, String::from_utf8_lossy(&output.stdout).to_string()))
}

fn run_and_check_git(args: &[&str]) -> Result<String, anyhow::Error> {
    let (status, output) = run_git(args)?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "Exit with non-zero status: git {}",
            args.join(" ")
        ));
    }
    Ok(output)
}

pub fn do_sync(config: &Config) -> Result<(), anyhow::Error> {
    let _guard = DirGuard::new(config.root_path())?;
    let status_output = run_and_check_git(&["status"])?;
    let dirty = status_output.contains("modified");
    if dirty {
        // the local status is dirty. stash them before pulling.
        run_and_check_git(&["stash"])?;
    }
    run_and_check_git(&["checkout", "master"])?;
    run_and_check_git(&["pull", "--rebase", "origin", "master"])?;
    run_and_check_git(&["push", "origin", "master"])?;
    if dirty {
        // restore stashed states.
        run_and_check_git(&["stash", "pop"])?;
    }
    Ok(())
}

mod command;
mod config;
mod entry;
mod walk;

use std::convert::From;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, info};
use structopt::StructOpt;

use anyhow::{anyhow, Context, Error};

use crate::config::Config;
use crate::walk::Walk;

#[derive(Debug)]
struct DotfilesPath(String);

impl DotfilesPath {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<Path> for DotfilesPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl AsRef<str> for DotfilesPath {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Default for DotfilesPath {
    fn default() -> Self {
        let path = match dirs::home_dir() {
            Some(home) => home.join("dotfiles").to_string_lossy().into_owned(),
            None => "/dotfiles".into(),
        };
        Self(path)
    }
}

impl fmt::Display for DotfilesPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.as_ref())
    }
}

impl std::str::FromStr for DotfilesPath {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DotfilesPath(s.to_owned()))
    }
}

#[derive(Debug)]
struct User(String);

impl Default for User {
    fn default() -> Self {
        if let Ok(s) = Command::new("git").arg("config").arg("user.name").output() {
            let s = String::from_utf8_lossy(&s.stdout);
            return User(s.trim().to_string());
        }
        if let Ok(s) = env::var("USER") {
            return User(s);
        }
        User("".to_owned())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for User {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(User(s.to_owned()))
    }
}

#[derive(Debug, StructOpt)]
struct CloneOptions {
    #[structopt(default_value)]
    user: User,
    #[structopt(default_value = "dotfiles")]
    repo: String,
    #[structopt(short, long)]
    shallow: bool,
}

fn do_clone(path: &DotfilesPath, options: &CloneOptions) -> Result<(), Error> {
    let github_url = format!("git@github.com:{}/{}", options.user, options.repo);
    info!("execute command: git clone {} {}", github_url, path);
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg(&github_url).arg(path.as_str());
    if options.shallow {
        cmd.arg("--depth=1");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!("failed to clone the repository: {}", status));
    }
    Ok(())
}

struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    fn new<P: AsRef<Path>>(in_dir: P) -> Result<Self, Error> {
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

#[derive(Debug, StructOpt)]
enum GitOptions {
    #[structopt(external_subcommand)]
    Args(Vec<String>),
}

fn do_git(path: &DotfilesPath, options: &GitOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(path)?;
    let GitOptions::Args(ref args) = *options;
    let status = Command::new("git").args(args).status()?;
    if !status.success() {
        return Err(anyhow!("failed to exec the git command: {}", status));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
// Edit a file
struct EditOptions {
    filename: String,
}

fn do_edit(path: &DotfilesPath, options: &EditOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(path)?;
    let editor = env::var("EDITOR").context("Failed to fetch $EDITOR")?;
    let status = Command::new(&editor).arg(&options.filename).status()?;
    if !status.success() {
        return Err(anyhow!("Failed to open editor {}: {}", editor, status));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
struct CommitOptions {
    #[structopt(long, short)]
    messege: Option<String>,
}

fn do_commit(path: &DotfilesPath, options: &CommitOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(path)?;
    let status = if let Some(ref message) = options.messege {
        let args = &["commit", "-A", "-m", message];
        info!("execute command: git commit -A -m {:?}", message);
        Command::new("git").args(args).status()?
    } else {
        let args = &["commit", "-A"];
        info!("execute command: git commit -A");
        Command::new("git").args(args).status()?
    };
    if !status.success() {
        return Err(anyhow!("Failed to commit: {}", status));
    }
    Ok(())
}

fn do_list(config: &Config) -> Result<(), Error> {
    let walk = Walk::new(config.root_path(), config.home_dir())?;
    for p in walk {
        let p = p?;
        println!("{}", p.display_relative());
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Clone your dotfiles repository
    Clone(CloneOptions),
    /// Execute git command in dotfiles directory
    Git(GitOptions),
    /// Edit your files
    Edit(EditOptions),
    /// Add all dirty files, then create a commit
    Commit(CommitOptions),
    /// List target files
    List,
    /// Show dotfiles status
    Status,
    /// Sync local and remote dotfiles
    Sync,
    /// Link dotfiles
    Link,
    /// Clean symbolic links
    Clean,
}

#[derive(Debug, StructOpt)]
struct DotfmCommand {
    #[structopt(name = "path", long, short, default_value)]
    path: DotfilesPath,

    #[structopt(subcommand)]
    sub_command: SubCommand,
}

fn run(command: &DotfmCommand) -> Result<(), Error> {
    let config = Config::new(command.path.as_ref(), dirs::home_dir().unwrap_or_default());

    use SubCommand::*;
    match command.sub_command {
        Clone(ref clone_opts) => do_clone(&command.path, clone_opts),
        Git(ref git_opts) => do_git(&command.path, git_opts),
        Edit(ref edit_opts) => do_edit(&command.path, edit_opts),
        Commit(ref commit_opts) => do_commit(&command.path, commit_opts),
        List => do_list(&config),
        Status => command::do_status(&config),
        Sync => todo!(),
        Link => command::do_link(&config),
        Clean => command::do_clean(&config),
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let cmd = DotfmCommand::from_args();
    debug!("{:?}", cmd);
    run(&cmd)
}

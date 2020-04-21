use std::convert::From;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::debug;
use structopt::StructOpt;

#[derive(Debug)]
pub enum Error {
    CommandFailed(&'static str, std::process::ExitStatus),
    Io(io::Error),
    Dyn(Box<dyn std::error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            CommandFailed(s, status) => write!(f, "{}: {}", s, status),
            Io(err) => write!(f, "{}", err),
            Dyn(err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::Dyn(value)
    }
}

#[derive(Debug)]
struct DotfilesPath(String);

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
        if let Ok(s) = std::env::var("USER") {
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
}

fn do_clone(path: &DotfilesPath, options: &CloneOptions) -> Result<(), Error> {
    let github_url = format!("git@github.com:{}/{}", options.user, options.repo);
    let status = Command::new("git")
        .arg("clone")
        .arg(&github_url)
        .arg(path.0.as_str())
        .status()?;
    if !status.success() {
        return Err(Error::CommandFailed(
            "failed to clone the repository",
            status,
        ));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
struct GitOptions {
    args: Vec<String>,
}

struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    fn new<P: AsRef<Path>>(in_dir: P) -> Result<Self, Error> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(in_dir.as_ref())?;
        Ok(Self { original })
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(self.original.as_path())
            .expect("failed to reset current directory")
    }
}

fn do_git(path: &DotfilesPath, options: &GitOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(&path.0)?;
    let status = Command::new("git").args(&options.args).status()?;
    if !status.success() {
        return Err(Error::CommandFailed(
            "failed to exec the git command",
            status,
        ));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    Clone(CloneOptions),
    Git(GitOptions),
}

#[derive(Debug, StructOpt)]
struct DotfmCommand {
    #[structopt(name = "path", long, short, default_value)]
    path: DotfilesPath,

    #[structopt(subcommand)]
    sub_command: SubCommand,
}

fn run(command: &DotfmCommand) -> Result<(), Error> {
    match command.sub_command {
        SubCommand::Clone(ref clone_opts) => do_clone(&command.path, clone_opts),
        SubCommand::Git(ref git_opts) => do_git(&command.path, git_opts),
    }
}

fn main() {
    env_logger::init();
    let cmd = DotfmCommand::from_args();
    debug!("{:?}", cmd);
    if let Err(err) = run(&cmd) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

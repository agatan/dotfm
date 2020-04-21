use std::convert::From;
use std::env;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{debug, info};
use structopt::StructOpt;

#[derive(Debug)]
pub enum Error {
    CommandFailed(&'static str, std::process::ExitStatus),
    Io(io::Error),
    Env(&'static str, env::VarError),
    Dyn(Box<dyn std::error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            CommandFailed(ref s, ref status) => write!(f, "{}: {}", s, status),
            Io(ref err) => write!(f, "{}", err),
            Env(s, ref err) => write!(f, "failed to get ${}: {}", s, err),
            Dyn(ref err) => write!(f, "{}", err),
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
}

fn do_clone(path: &DotfilesPath, options: &CloneOptions) -> Result<(), Error> {
    let github_url = format!("git@github.com:{}/{}", options.user, options.repo);
    info!("execute command: git clone {} {}", github_url, path.0);
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

fn do_git(path: &DotfilesPath, options: &[String]) -> Result<(), Error> {
    assert_eq!(options[0], "git");
    let _guard = DirGuard::new(&path.0)?;
    let status = Command::new("git").args(&options[1..]).status()?;
    if !status.success() {
        return Err(Error::CommandFailed(
            "failed to exec the git command",
            status,
        ));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
struct EditOptions {
    filename: String,
}

fn do_edit(path: &DotfilesPath, options: &EditOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(&path.0)?;
    let editor = env::var("EDITOR").map_err(|err| Error::Env("EDITOR", err))?;
    let status = Command::new(editor).arg(&options.filename).status()?;
    if !status.success() {
        return Err(Error::CommandFailed("failed to edit the file", status));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
/// Add all dirty files, then create a commit.
struct CommitOptions {
    #[structopt(long, short)]
    messege: Option<String>,
}

fn do_commit(path: &DotfilesPath, options: &CommitOptions) -> Result<(), Error> {
    let _guard = DirGuard::new(&path.0)?;
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
        return Err(Error::CommandFailed("failed to commit", status));
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    Clone(CloneOptions),
    #[structopt(external_subcommand)]
    Git(Vec<String>),
    Edit(EditOptions),
    Commit(CommitOptions),
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
        SubCommand::Edit(ref edit_opts) => do_edit(&command.path, edit_opts),
        SubCommand::Commit(ref commit_opts) => do_commit(&command.path, commit_opts),
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

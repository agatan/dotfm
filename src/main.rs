use std::fmt;
use std::process::Command;
use structopt::StructOpt;

#[derive(Debug)]
pub enum Error {
    BaseDirUnspecified,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            BaseDirUnspecified => f.write_str("dotfiles base directory is unspecified"),
        }
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

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(name = "clone")]
    Clone(CloneOptions),
}

#[derive(Debug, StructOpt)]
struct DotfmCommand {
    #[structopt(name = "path", long, short, default_value)]
    path: DotfilesPath,

    #[structopt(subcommand)]
    sub_command: SubCommand,
}

fn main() {
    let cmd = DotfmCommand::from_args();
    println!("{:?}", cmd);
}

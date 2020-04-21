use std::convert::From;
use std::env;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    CommandFailed(&'static str, std::process::ExitStatus),
    Io(io::Error),
    Env(&'static str, env::VarError),
    Ignore(ignore::Error),
    Dyn(Box<dyn std::error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            CommandFailed(ref s, ref status) => write!(f, "{}: {}", s, status),
            Io(ref err) => write!(f, "{}", err),
            Env(s, ref err) => write!(f, "failed to get ${}: {}", s, err),
            Ignore(ref err) => write!(f, "{}", err),
            Dyn(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<ignore::Error> for Error {
    fn from(value: ignore::Error) -> Self {
        Error::Ignore(value)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::Dyn(value)
    }
}

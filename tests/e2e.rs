use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;

fn command() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn test_help() {
    command().arg("--help").assert().success();
}

#[test]
fn test_git_status() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles = temp.child("dotfiles");
    dotfiles.create_dir_all().unwrap();
    let status = Command::new("git")
        .arg("init")
        .arg(dotfiles.path())
        .status()
        .unwrap();
    assert!(status.success());
    command()
        .arg("-p")
        .arg(dotfiles.path())
        .arg("git")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_list() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles = temp.child("dotfiles");
    dotfiles.create_dir_all().unwrap();
    let status = Command::new("git")
        .arg("init")
        .arg(dotfiles.path())
        .status()
        .unwrap();
    assert!(status.success());
    dotfiles
        .child(".dotfmignore")
        .write_str(".must_be_ignored")
        .unwrap();
    dotfiles.child(".must_be_ignored").touch().unwrap();
    dotfiles.child(".vimrc").touch().unwrap();
    command()
        .arg("-p")
        .arg(dotfiles.path())
        .arg("list")
        .assert()
        .stdout(predicates::str::is_match(".vimrc").unwrap())
        .success();
}

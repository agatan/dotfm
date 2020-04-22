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

fn test_clone_command(path: &std::path::Path) -> Command {
    let mut cmd = command();
    cmd.arg("-p")
        .arg(path)
        .arg("clone")
        .arg("agatan")
        .arg("dotfiles")
        .arg("--shallow");
    cmd
}

#[test]
fn test_clone() {
    let temp = assert_fs::TempDir::new().unwrap();
    let clone_into = temp.child("dotfiles");
    test_clone_command(clone_into.path()).assert().success();
}

#[test]
fn test_git_status() {
    let temp = assert_fs::TempDir::new().unwrap();
    let clone_into = temp.child("dotfiles");
    test_clone_command(clone_into.path()).assert().success();
    command()
        .arg("-p")
        .arg(clone_into.path())
        .arg("git")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_list() {
    let temp = assert_fs::TempDir::new().unwrap();
    let clone_into = temp.child("dotfiles");
    test_clone_command(clone_into.path()).assert().success();
    command()
        .arg("-p")
        .arg(clone_into.path())
        .arg("list")
        .assert()
        .stdout(predicates::str::contains(".vimrc"))
        .success();
}

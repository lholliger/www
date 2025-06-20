use std::process::Command;

fn main() {
    // note: add error checking yourself.
    let git_hash = String::from_utf8(Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_time = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%ct", "HEAD"]).output().unwrap().stdout).unwrap();
    let git_message = String::from_utf8(Command::new("git").args(&["show", "--no-patch", "--format=%B", "HEAD"]).output().unwrap().stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_TIME={}", git_time);
    println!("cargo:rustc-env=GIT_MESSAGE={}", git_message.trim());
}
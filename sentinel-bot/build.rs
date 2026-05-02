use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let git_hash_output = Command::new("git")
        .args(&["rev-parse", "--short=8", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(git_hash_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let git_branch_output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    let git_branch = String::from_utf8(git_branch_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);

    let build_time_seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    println!("cargo:rustc-env=BUILD_TIME_SEC={}", build_time_seconds);
}

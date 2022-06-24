use std::{env::temp_dir, process::Command};

use tempfile::tempfile;

const BIN_PATH: &str = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));

#[test]
fn an_executable_named_after_the_package_exits_with_zero() {
    let status = Command::new(BIN_PATH).status().unwrap();
    assert!(status.success())
}

#[test]
fn if_persistance_file_doesnt_exist_it_is_created() {
    let dir = temp_dir();
    let persistance_file_path = dir.join("pf");
    let status = Command::new(BIN_PATH)
        .env("PERSISTANCE_FILE", persistance_file_path.clone())
        .status()
        .unwrap();
    assert!(status.success());
    assert!(persistance_file_path.exists());
}

#[test]
fn if_persistance_file_exists_it_is_unaltered() {
    let persistance_file = tempfile().unwrap();
    let status = Command::new(BIN_PATH)
        .env("PERSISTANCE_FILE", persistance_file.)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(persistance_file.exists());
}

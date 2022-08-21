use std::{path::PathBuf, process::Command};

use tempfile::tempdir;

const BIN_PATH: &str = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));

fn make_command_sans_persistance_file() -> (PathBuf, Command) {
    let temp_dir_path = tempdir().unwrap().into_path();
    let persistance_file_path = temp_dir_path.join("pf");

    let mut command = Command::new(BIN_PATH);
    command.env("PERSISTANCE_FILE", &persistance_file_path);

    (persistance_file_path, command)
}

#[test]
fn an_executable_named_after_the_package_exits_with_zero() {
    let (_, mut command) = make_command_sans_persistance_file();
    assert!(command.status().unwrap().success());
}

#[test]
fn if_persistance_file_doesnt_exist_it_is_created() {
    let (persistance_file_path, mut command) = make_command_sans_persistance_file();
    let status = command.status().unwrap();
    assert!(status.success());
    assert!(persistance_file_path.exists());
}

#[test]
fn if_persistance_file_env_not_defined_exit_with_non_zero() {
    let status = Command::new(BIN_PATH).status().unwrap();
    assert_ne!(status.code().unwrap(), 0);
}

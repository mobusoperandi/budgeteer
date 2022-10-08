use std::{path::PathBuf, process};

use tempfile::{tempdir, TempDir};

const BIN_PATH: &str = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));
const PERSISTENCE_FILE: &str = "PERSISTENCE_FILE";

fn path_to_non_existant_file() -> (PathBuf, TempDir) {
    // return a Tuple of both the path and the directory handle
    let temp_dir = tempdir().unwrap();
    (temp_dir.path().join("non_existant_file"), temp_dir)
}

#[test]
fn if_persistence_file_env_not_defined_exit_with_non_zero() {
    let status = process::Command::new(BIN_PATH)
        .stderr(process::Stdio::null())
        .status()
        .unwrap();
    assert_ne!(status.code().unwrap(), 0);
}

#[test]
fn readme() {
    let (persistence_file_path, temp_dir) = path_to_non_existant_file();
    trycmd::TestCases::new()
        .case("README.md")
        .env(PERSISTENCE_FILE, persistence_file_path.to_str().unwrap());
    drop(temp_dir)
}

use std::process::Command;

const BIN_PATH: &str = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));

#[test]
fn an_executable_named_after_the_package_exits_with_zero() {
    let status = Command::new(BIN_PATH).status().unwrap();
    assert!(status.success())
}

#[test]
fn if_persistance_file_doesnt_exist_it_is_created() {
    process::Command::new(program)
}

#[test]
fn if_persistance_file_exists_it_is_unaltered() {}

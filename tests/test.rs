use std::process;

#[test]
fn an_executable_named_after_the_package_exits_with_zero() {
    let bin_path = env!(concat!("CARGO_BIN_EXE_", env!("CARGO_PKG_NAME")));
    let status = process::Command::new(bin_path).status().unwrap();
    assert!(status.success())
}

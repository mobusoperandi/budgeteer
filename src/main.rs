#![doc = include_str!("../README.md")]
use std::{env, fs};

fn main() {
    let persistence_file_path = env::var("PERSISTENCE_FILE").unwrap();

    fs::File::create(persistence_file_path).unwrap();
}

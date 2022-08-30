#![doc = include_str!("../README.md")]
#![feature(lint_reasons)]
mod account;
mod events;
mod sum;
use std::{env, fs};

fn main() {
    let persistence_file_path = env::var("PERSISTENCE_FILE").unwrap();

    fs::File::create(persistence_file_path).unwrap();
}

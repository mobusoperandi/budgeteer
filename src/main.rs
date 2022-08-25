use std::{env, fs};

fn main() {
    let persistance_file_path = env::var("PERSISTANCE_FILE").unwrap();

    fs::File::create(persistance_file_path).unwrap();
}

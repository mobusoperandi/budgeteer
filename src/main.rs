#![doc = include_str!("../README.md")]

mod arguments_interpreter;
mod cli;
mod entities {
    pub(crate) mod account;
    pub(crate) mod amount;
    pub(crate) mod balance;
    pub(crate) mod move_;
    pub(crate) mod sum;
    pub(crate) mod transaction;
    pub(crate) mod unit;
}
mod events;
mod reports;
mod views;

use anyhow::Result;
use clap::Parser;
use std::{
    env, fs,
    io::{Seek, Write},
    path::PathBuf,
};

use events::{Event, Events};

fn main() {
    let args_os = std::env::args_os();
    // TODO see whether all the validation can be done here
    let arguments = cli::Arguments::try_parse_from(args_os).unwrap();
    // TODO introduce struct for return type
    let arguments_interpreter::Actions { event, report } =
        arguments_interpreter::interpret(arguments).unwrap();
    // TODO default persistence file path
    let persistence_file_path = PathBuf::from(env::var("PERSISTENCE_FILE").unwrap());
    let mut file_options = fs::OpenOptions::new();
    file_options.read(true).write(true);
    let mut persistence_file = file_options
        .open(&persistence_file_path)
        .or_else(|_| -> Result<_> {
            let mut new_persistence_file =
                file_options.create(true).open(&persistence_file_path)?;
            new_persistence_file.write_all(ron::to_string(&Vec::<Event>::new())?.as_bytes())?;
            new_persistence_file.rewind()?;
            Ok(new_persistence_file)
        })
        .unwrap();
    let mut events = Events::try_from_reader(&mut persistence_file).unwrap();
    if let Some(event) = event {
        events.try_push(event).unwrap();
        persistence_file.rewind().unwrap();
        persistence_file.set_len(0).unwrap();
        events.try_write(&mut persistence_file).unwrap();
    }
    if let Some(report) = report {
        let report = report.compile(&events).unwrap();
        print!("{report}");
    }
}

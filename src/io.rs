use std::{
    fs,
    io::{self, Read, Write},
};

pub(crate) trait Io<F, S>
where
    F: Read + Write,
    S: Write,
{
    fn persistance_file() -> F;
    fn stdout() -> S;
}

pub(crate) struct RealIo {
    persistance_file: fs::File,
    stdout: io::Stdout,
}

impl RealIo {
    pub(crate) fn new(persistance_file: fs::File, stdout: io::Stdout) -> Self {
        Self {
            persistance_file,
            stdout,
        }
    }
}

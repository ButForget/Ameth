pub mod root;

use clap::Error as ClapError;
use std::ffi::OsString;

#[derive(Debug)]
pub enum Error {
    Clap(ClapError),
    Runtime(String),
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Error> {
    root::run(args)
}

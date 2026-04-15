pub mod root;

use std::ffi::OsString;

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    root::run(args)
}

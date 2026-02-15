use std::path::PathBuf;

use crate::error::Error;

include!(concat!(env!("OUT_DIR"), "/_.rs"));

pub mod client;
pub mod error;
pub mod server;

#[macro_use]
extern crate log;

pub fn socket_path() -> Result<PathBuf, Error> {
    let mut runtime_dir = dirs::runtime_dir().ok_or(Error::XdgRuntimeDirNotSet)?;
    runtime_dir.push("minecraftd.sock");
    Ok(runtime_dir)
}

use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Toni Peter")]
pub struct Args {
    pub pgn_file: PathBuf,
}

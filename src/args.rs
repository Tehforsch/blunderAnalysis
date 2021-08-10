use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Toni Peter")]
pub struct Args {
    pub database: PathBuf,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clap, Debug)]
pub enum Command {
    Scan(ScanOpts),
    Show,
}

#[derive(Clap, Debug, Clone)]
pub struct ScanOpts {
    pub pgn_file: PathBuf,
    pub num_moves: usize,
    pub player_name: String,
}

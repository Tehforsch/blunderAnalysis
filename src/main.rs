pub mod args;

use std::{error::Error, fs};

use clap::Clap;
use pgnparse::parser::*;
use uci::Engine;

use crate::args::Args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let pgns_string = fs::read_to_string(&args.pgn_file)?;
    let game_pgns = split_pgns_into_games(&pgns_string);

    for game_pgn in game_pgns {
        let result = parse_pgn_to_rust_struct(game_pgn);
        dbg!(&result);
    }

    // let engine = Engine::new("stockfish").unwrap();
    // println!("{:?}", engine.bestmove());
    Ok(())
}

fn split_pgns_into_games<'a>(pgns: &'a String) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(pgns.split("\n\n\n").into_iter())
}

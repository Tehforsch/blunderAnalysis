pub mod analysis;
pub mod args;
pub mod blunder;
pub mod config;
pub mod database;
pub mod engine;
pub mod evaluation;
pub mod game;

use std::fs;
use std::path::Path;
use std::time::Duration;

use analysis::{AnalysisThread, AnalysisThreadHandle};
use anyhow::Result;
use args::Command;
use blunder::Blunder;
use clap::Clap;
use config::NUM_THREADS;
use counter::Counter;
use database::Database;

use crate::args::{Args, ScanOpts};

#[derive(PartialEq, Eq)]
enum Color {
    White,
    Black,
}

impl Color {
    fn to_play(&self, move_num: usize) -> bool {
        let color = match move_num.rem_euclid(2) {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Impossible value"),
        };
        &color == self
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let database = Database::read(&args.database)?;
    match args.command {
        Command::Scan(opts) => scan(database, &args.database, opts),
        Command::Show => show_blunders(database),
    }
}

fn scan(mut database: Database, database_path: &Path, opts: ScanOpts) -> Result<()> {
    let pgns_string = fs::read_to_string(&opts.pgn_file)?;
    let mut game_pgns = split_pgns_into_games(&pgns_string);
    let mut threads: Vec<AnalysisThreadHandle> = vec![];
    loop {
        if threads.len() < NUM_THREADS {
            if let Some(game_str) = game_pgns.next() {
                threads.push(AnalysisThread::start(game_str, &opts));
            }
        }
        if threads.is_empty() {
            break;
        }
        for mut thread in threads.iter_mut() {
            let received_result = thread.receiver.recv_timeout(Duration::from_millis(20));
            if let Ok(game) = received_result {
                println!("Finished analyzing {}", game.id);
                database.add_game(game);
                database.write(database_path)?;
                thread.finished = true;
            }
        }
        let (finished_threads, running_threads): (Vec<_>, Vec<_>) =
            threads.into_iter().partition(|thread| thread.finished);
        for finished_thread in finished_threads.into_iter() {
            finished_thread.handle.join().unwrap().unwrap();
        }
        threads = running_threads;
    }
    Ok(())
}

fn split_pgns_into_games<'a>(pgns: &'a str) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(pgns.split("\n\n\n"))
}

fn show_blunders(database: Database) -> Result<()> {
    let all_blunders: Vec<&Blunder> = database
        .games
        .iter()
        .flat_map(|game| game.blunders.iter())
        .collect();
    let counter = all_blunders.iter().collect::<Counter<_>>();
    for (blunder, count) in counter.iter() {
        if *count > 1 {
            println!(
                "In position: {}\n you played {}",
                blunder.position, blunder.move_
            );
        }
    }
    Ok(())
}

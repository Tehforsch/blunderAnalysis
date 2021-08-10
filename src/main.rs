pub mod analysis;
pub mod args;
pub mod blunder;
pub mod config;
pub mod database;
pub mod engine;
pub mod evaluation;
pub mod game;
pub mod game_info;

use std::fs;
use std::time::Duration;

use analysis::{AnalysisThread, AnalysisThreadHandle};
use anyhow::Result;
use args::Command;
use blunder::Blunder;
use clap::Clap;
use config::NUM_THREADS;
use counter::Counter;
use database::Database;
use game::Game;
use game_info::GameInfo;
use pgnparse::parser::parse_pgn_to_rust_struct;

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
    let mut database = Database::read(&args.database)?;
    match args.command {
        Command::Scan(opts) => scan(&mut database, opts),
        Command::Show => show_blunders(database),
    }
}

fn scan(database: &mut Database, opts: ScanOpts) -> Result<()> {
    let pgns_string = fs::read_to_string(&opts.pgn_file)?;
    let mut games: Vec<GameInfo> = split_pgns_into_games(&pgns_string);
    let (seen_games, unseen_games): (Vec<_>, Vec<_>) = games
        .into_iter()
        .partition(|game| database.game_exists(game));
    println!(
        "Skipping {} games that have already been analyzed.",
        seen_games.len()
    );
    games = unseen_games;
    run_scans(database, opts, games)
}

fn run_scans(database: &mut Database, opts: ScanOpts, games: Vec<GameInfo>) -> Result<()> {
    let mut threads: Vec<AnalysisThreadHandle> = vec![];
    let num_games = games.len();
    let mut games = games.into_iter().enumerate();
    loop {
        if threads.len() < NUM_THREADS {
            if let Some((num, game_info)) = games.next() {
                println!("Analyzing {} / {}", num, num_games);
                threads.push(AnalysisThread::start(game_info, &opts));
            }
        }
        if threads.is_empty() {
            break;
        }
        for mut thread in threads.iter_mut() {
            let received_result = thread.receiver.recv_timeout(Duration::from_millis(20));
            if let Ok(game) = received_result {
                handle_finished_analysis(database, game)?;
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
    println!("Finished analyzing.");
    Ok(())
}

fn handle_finished_analysis(database: &mut Database, game: Game) -> Result<()> {
    println!("Finished analyzing {}", game.id);
    database.add_game(game);
    database.write()
}

fn split_pgns_into_games(pgns: &str) -> Vec<GameInfo> {
    pgns.split("\n\n\n")
        .map(parse_pgn_to_rust_struct)
        .filter(|game| !game.moves.is_empty())
        .map(|info| GameInfo { info })
        .collect()
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

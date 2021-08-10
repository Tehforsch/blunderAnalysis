pub mod args;
pub mod engine;
pub mod database;
pub mod blunder;
pub mod game;
pub mod evaluation;

use std::fs;
use std::path::Path;

use counter::Counter;
use anyhow::Result;
use args::Command;
use blunder::Blunder;
use database::Database;
use game::Game;
use regex::Regex;
use clap::Clap;
use pgnparse::parser::*;

use crate::args::{Args, ScanOpts};
use crate::engine::Engine;
use crate::evaluation::Evaluation;

const NUM_MOVES_TO_SKIP: usize = 5;
const BLUNDER_CENTIPAWN_LOSS: i32 = 40;
const DEFAULT_DEPTH: usize = 12;
const ACCURATE_DEPTH: usize = 25;

#[derive(PartialEq, Eq)]
enum Color {
    White,
    Black
}

impl Color {
    fn to_play(&self, move_num: usize) -> bool {
        let color = match move_num.rem_euclid(2) {
            0 => Color::White,
            1 => Color::Black,
            _ => panic!("Impossible value")
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
    let game_pgns = split_pgns_into_games(&pgns_string);
    for game_pgn in game_pgns {
        let game_info = parse_pgn_to_rust_struct(game_pgn);
        let id = get_game_id(&game_info);
        if database.games.iter().find(|game| game.id == id).is_some() {
            continue;
        }
        println!("Scanning game {}", id);
        let blunders = find_blunders(&game_info, opts.num_moves, &opts.player_name)?;
        database.add_game(Game { id, blunders });
        database.write(database_path)?;
    }
    Ok(())
}

fn find_blunders(result: &PgnInfo, num_moves: usize, player_name: &str) -> Result<Vec<Blunder>> {
    let player_color = get_player_color(result, player_name);
    let engine_1 = Engine::new("stockfish")?;
    let engine_2 = Engine::new("stockfish")?;
    let mut blunders = vec![];
    for (move_num, chess_move) in result.moves.iter().take(num_moves).enumerate() {
        if !player_color.to_play(move_num) || move_num < NUM_MOVES_TO_SKIP {
            continue
        }
        let (_, eval_before) = get_evaluation_from_position(&engine_1, &chess_move.fen_before, DEFAULT_DEPTH)?;
        let (_, eval_after) = get_evaluation_from_position(&engine_2, &chess_move.fen_after, DEFAULT_DEPTH)?;
        let centipawn_loss = eval_before.0 + eval_after.0;
        if centipawn_loss > BLUNDER_CENTIPAWN_LOSS {
            let (m1, eval_before) = get_evaluation(&engine_1, ACCURATE_DEPTH)?;
            let (_, eval_after) = get_evaluation(&engine_2, ACCURATE_DEPTH)?;
            let centipawn_loss = eval_before.0 + eval_after.0;
            if centipawn_loss > BLUNDER_CENTIPAWN_LOSS {
                println!("In position: {}\n you played {} and lost {} ({} vs {}). Instead, you should have played {}.", chess_move.fen_before, &chess_move.san, centipawn_loss, &eval_before.0, -&eval_after.0, &m1);
                blunders.push(Blunder {
                    position: chess_move.fen_before.clone(),
                    move_: chess_move.san.clone(),
                    eval_before,
                    eval_after,
                })
            }
        }
    }
    Ok(blunders)
}

fn get_evaluation_from_position(engine: &Engine, fen: &str, depth: usize) -> Result<(String, Evaluation)> {
    engine.set_position(&fen)?;
    get_evaluation(engine, depth)
}

fn get_evaluation(engine: &Engine, depth: usize) -> Result<(String, Evaluation)> {
    let analysis = engine.run(depth)?;
    let best_move = get_best_move(&analysis);
    Ok((best_move, get_evaluation_from_string(&analysis)))
}

fn get_best_move(analysis: &str) -> String {
    let re = Regex::new("bestmove ([-a-z0-9]*)")
        .unwrap();
    let captures = re.captures_iter(&analysis);
    let capture = captures.last().unwrap();
    capture.get(1).unwrap().as_str().into()
}

fn get_player_color(result: &PgnInfo, player_name: &str) -> Color {
    if result.headers["White"] == player_name {
        Color::White
    }
    else {
        assert_eq!(result.headers["Black"], player_name);
        Color::Black
    }
}

fn get_game_id(game_pgn: &PgnInfo) -> String {
    game_pgn.headers["Site"].clone()
}


fn get_evaluation_from_string(analysis: &str) -> Evaluation {
    let re = Regex::new("score (cp|mate) ([-0-9]*)")
        .unwrap();
    let captures = re.captures_iter(&analysis);
    let capture = captures.last().unwrap();
    let score_type: &str = capture.get(1).unwrap().as_str();
    let value: i32 = capture.get(2).unwrap().as_str().parse().unwrap();
    match score_type {
        "cp" => Evaluation(value),
        "mate" => Evaluation(value * 2000),
        _ => panic!("Unkown score: {}", analysis),
    }
}

fn split_pgns_into_games<'a>(pgns: &'a String) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(pgns.split("\n\n\n").into_iter())
}

fn show_blunders(database: Database) -> Result<()> {
    let all_blunders: Vec<&Blunder> = database.games.iter().flat_map(|game| game.blunders.iter()).collect();
    let counter = all_blunders.iter().collect::<Counter<_>>();
    for (blunder, count) in counter.iter() {
        if *count > 1 {
            println!("In position: {}\n you played {}", blunder.position, blunder.move_);
        }
    }
    Ok(())
}

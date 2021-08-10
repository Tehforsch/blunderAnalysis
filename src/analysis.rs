use anyhow::Result;
use pgnparse::parser::{PgnInfo, parse_pgn_to_rust_struct};

use std::thread::JoinHandle;
use std::{thread};
use std::sync::mpsc::{Receiver, Sender, channel};
use regex::Regex;

use crate::Color;
use crate::args::ScanOpts;
use crate::config::{ACCURATE_DEPTH, BLUNDER_CENTIPAWN_LOSS, DEFAULT_DEPTH, NUM_MOVES_TO_SKIP};
use crate::game::Game;
use crate::blunder::Blunder;
use crate::engine::Engine;
use crate::evaluation::Evaluation;

pub struct AnalysisThreadHandle {
    pub handle: JoinHandle<Result<()>>,
    pub receiver: Receiver<Game>,
    pub finished: bool,
}

pub struct AnalysisThread {
    sender: Sender<Game>,
}

impl AnalysisThread {
    pub fn start<'a>(game_pgn: &'a str, scan_opts: &ScanOpts) -> AnalysisThreadHandle {
        let (result_sender, result_receiver) = channel();
        let analysis_thread = AnalysisThread {sender: result_sender};
        let scan_opts_cloned = scan_opts.clone();
        let game_pgn_cloned = game_pgn.to_owned();
        let handle = thread::spawn(move || {
            analysis_thread.run(&game_pgn_cloned, &scan_opts_cloned)
        });
        AnalysisThreadHandle {
            handle,
            receiver: result_receiver,
            finished: false,
        }
    }

    fn run(self, game_pgn: &str, scan_opts: &ScanOpts) -> Result<()> {
        let game_info = parse_pgn_to_rust_struct(game_pgn);
        let id = get_game_id(&game_info);
        let blunders = find_blunders(&game_info, scan_opts.num_moves, &scan_opts.player_name)?;
        self.sender.send(Game {
            id,
            blunders
        }).unwrap();
        Ok(())
    }
}

fn get_game_id(game_pgn: &PgnInfo) -> String {
    game_pgn.headers["Site"].clone()
}

pub fn find_blunders(result: &PgnInfo, num_moves: usize, player_name: &str) -> Result<Vec<Blunder>> {
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

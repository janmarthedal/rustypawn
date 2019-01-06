extern crate rand;
use std::io;

use rand::prelude::{thread_rng, Rng};
use rustypawn::Game;
use rustypawn::Move;
use rustypawn::algebraic_to_move;

fn legal_moves(game: &mut Game) -> Vec<Move> {
    let mut result: Vec<Move> = Vec::new();
    let move_list = game.generate_moves();
    for mv in &move_list {
        match game.make_move(mv) {
            Some(umv) => {
                game.unmake_move(mv, umv);
                result.push(mv.clone());
            },
            None => {}
        }
    }
    result
}

fn make_move_algebraic(game: &mut Game, input_move: &str) {
    let input_move = algebraic_to_move(input_move);
    let moves = legal_moves(game);
    for mv in &moves {
        if *mv == input_move {
            game.make_move(mv);
            return;
        }
    }
    panic!("make_move_algebraic");
}

fn think(game: &mut Game) -> Option<Move> {
    let moves = legal_moves(game);
    if moves.is_empty() {
        return None;
    }
    let num = thread_rng().gen_range(0, moves.len());
    Some(moves[num].clone())
}

fn main() {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0").unwrap();
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let args: Vec<&str> = input.trim().split_whitespace().collect();
                match args[0] {
                    "uci" => {
                        println!("id name rustypawn");
                        println!("id author Jan Marthedal Rasmussen");
                        println!("uciok");
                    },
                    "isready" => {
                        println!("readyok");
                    },
                    "position" => {
                        if args.len() < 2 {
                            panic!("Missing arguments to 'position'");
                        }
                        let mut index = 2;
                        let fen = match args[1] {
                            "startpos" => {
                                // position startpos
                                // position startpos moves e2e4 e7e5
                                String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0")
                            },
                            "fen" => {
                                // position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w moves e2e4 e7e5
                                if args.len() < 4 {
                                    panic!("Too few 'position fen' arguments");
                                }
                                let mut fen_items: Vec<&str> = Vec::new();
                                while index < args.len() && args[index] != "moves" {
                                    fen_items.push(args[index]);
                                    index += 1;
                                }
                                fen_items.join(" ")
                            },
                            _ => panic!("Unknown 'position' argument '{}'", args[1])
                        };
                        // println!("init position: {}", fen);
                        game = match Game::from_fen(&fen[..]) {
                            Ok(g) => g,
                            Err(e) => panic!("Illegal fen string '{}' ({})", fen, e)
                        };
                        if index < args.len() && args[index] == "moves" {
                            index += 1;
                            while index < args.len() {
                                make_move_algebraic(&mut game, args[index]);
                                index += 1;
                            }
                        }
                    },
                    "go" => {
                        let mv = match think(&mut game) {
                            Some(m) => m,
                            None => panic!("No legal move")
                        };
                        println!("bestmove {}", mv.to_algebraic());
                    }
                    _ => continue
                }
            }
            Err(error) => panic!("error: {}", error),
        }
    }
}

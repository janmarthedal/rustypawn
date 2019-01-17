use std::io;

use rustypawn::Comms;
use rustypawn::Game;
use rustypawn::MAX_DEPTH;
use rustypawn::make_move_algebraic;
use rustypawn::think;

fn main() {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0").unwrap();
    let mut comms = Comms::new("comms.txt");

    println!("Rustypawn");
    println!("  quiesce: {}", if cfg!(noquiesce) { "no" } else { "yes"});

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let line = input.trim();
                comms.input(line);
                let mut arg_iter = line.split_whitespace();
                match arg_iter.next() {
                    Some("uci") => {
                        comms.output("id name rustypawn");
                        comms.output("id author Jan Marthedal Rasmussen");
                        comms.output("uciok");
                    },
                    Some("isready") => {
                        comms.output("readyok");
                    },
                    Some("position") => {
                        let fen = match arg_iter.next() {
                            Some("startpos") => {
                                // position startpos
                                // position startpos moves e2e4 e7e5
                                String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0")
                            },
                            Some("fen") => {
                                // position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w moves e2e4 e7e5
                                let mut fen_items: Vec<&str> = Vec::new();
                                loop {
                                    match arg_iter.next() {
                                        Some("moves") => break,
                                        Some(s) => fen_items.push(s),
                                        None => break
                                    }
                                }
                                fen_items.join(" ")
                            },
                            Some(_) => comms.fatal("Unknown 'position' argument"),
                            _ => comms.fatal("Missing argument to 'position'")
                        };
                        game = match Game::from_fen(&fen[..]) {
                            Ok(g) => g,
                            Err(e) => comms.fatal(format!("Illegal fen string '{}' ({})", fen, e))
                        };
                        loop {
                            match arg_iter.next() {
                                Some("moves") => continue,
                                Some(s) => make_move_algebraic(&mut game, s),
                                None => break
                            }
                        }
                    },
                    Some("go") => {
                        let white_to_move = game.white_to_move();
                        let mut millis_to_think: u64 = 10 * 60 * 1000;  // 10 minutes
                        let mut wtime: i32 = -1;
                        let mut btime: i32 = -1;
                        let mut movestogo: u64 = 0;
                        loop {
                            match arg_iter.next() {
                                Some("wtime") => {
                                    match arg_iter.next() {
                                        Some(s) => {
                                            wtime = match s.parse::<i32>() {
                                                Ok(n) => n,
                                                _ => comms.fatal("Error parsing wtime")
                                            };
                                            if white_to_move && movestogo > 0 {
                                                millis_to_think = wtime as u64 / movestogo;
                                            }
                                        },
                                        None => comms.fatal("Missing wtime")
                                    }
                                },
                                Some("btime") => {
                                    match arg_iter.next() {
                                        Some(s) => {
                                            btime = match s.parse::<i32>() {
                                                Ok(n) => n,
                                                _ => comms.fatal("Error parsing btime")
                                            };
                                            if !white_to_move && movestogo > 0 {
                                                millis_to_think = btime as u64 / movestogo;
                                            }
                                        },
                                        None => comms.fatal("Missing btime")
                                    }
                                },
                                Some("movestogo") => {
                                    match arg_iter.next() {
                                        Some(s) => {
                                            movestogo = match s.parse::<u64>() {
                                                Ok(n) => n,
                                                _ => comms.fatal("Error parsing movestogo")
                                            };
                                            if white_to_move && wtime > 0 {
                                                millis_to_think = wtime as u64 / movestogo;
                                            } else if !white_to_move && btime > 0 {
                                                millis_to_think = btime as u64 / movestogo;
                                            }
                                        },
                                        None => comms.fatal("Missing movestogo")
                                    }
                                },
                                Some("movetime") => {
                                    match arg_iter.next() {
                                        Some(s) => {
                                            millis_to_think = match s.parse::<u64>() {
                                                Ok(n) => n,
                                                _ => comms.fatal("Error parsing movetime")
                                            };
                                        },
                                        None => comms.fatal("Missing movetime")
                                    }
                                },
                                Some(s) => comms.debug(format!("Ignore go argument '{}'", s)),
                                None => break
                            }
                        }
                        let mv = match think(&mut game, millis_to_think, MAX_DEPTH, &mut comms) {
                            Some(m) => m,
                            None => comms.fatal("No legal move")
                        };
                        comms.output(format!("bestmove {}", mv.to_algebraic()));
                    }
                    _ => continue
                }
            }
            Err(error) => comms.fatal(format!("error: {}", error))
        }
    }
}

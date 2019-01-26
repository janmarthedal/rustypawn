use std::io;
use std::io::Write;
use std::fs::File;

use rustypawn::ThinkInfo;
use rustypawn::MoveTrait;
use rustypawn::Game;
use rustypawn::MAX_DEPTH;
use rustypawn::make_move_algebraic;
use rustypawn::think;

struct Comms {
    file: Option<File>
}

impl Comms {
    pub fn new(name: Option<&str>) -> Comms {
        Comms {
            file: match name {
                Some(n) => Some(File::create(n).unwrap()),
                None => None
            }
        }
    }
    fn write(self: &mut Comms, prefix: &str, msg: &str) {
        match &mut self.file {
            Some(f) => {
                f.write_all(prefix.as_bytes()).unwrap();
                f.write_all(msg.as_bytes()).unwrap();
                f.write_all(b"\n").unwrap();
            },
            None => {}
        }
    }
    pub fn input(self: &mut Comms, msg: &str) {
        self.write("> ", msg);
    }
    pub fn output<S: Into<String>>(self: &mut Comms, msg: S) {
        let s = msg.into();
        println!("{}", s);
        self.write("< ", &s[..]);
    }
    pub fn fatal<S: Into<String>>(self: &mut Comms, msg: S) -> ! {
        let s = msg.into();
        self.write("! ", &s[..]);
        panic!(s);
    }
    pub fn debug<S: Into<String>>(self: &mut Comms, msg: S) {
        let s = msg.into();
        self.write("- ", &s[..]);
    }
}

impl ThinkInfo for Comms {
    fn think_info(self: &mut Comms, depth: usize, score: isize, mate_in: isize, node_count: usize, millis: u64, moves: &Vec<String>) {
        let nps = if millis > 0 { 1000 * node_count as u64 / millis } else { 0 };
        let mate = if mate_in != 0 { format!(" mate {}", mate_in) } else { String::new() };
        let msg = format!("info depth {} score cp {}{} nodes {} time {} nps {} pv {}",
            depth, score, mate, node_count, millis, nps, moves.join(" "));
        self.output(msg);
    }
}

fn main() {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0").unwrap();
    let mut comms = Comms::new(None);
    // let mut comms = Comms::new("/tmp/rustypawn-0.4.log");

    println!("Rustypawn");

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

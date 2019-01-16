use std::fs::File;
use std::io::Write;
use std::io;
use std::time::Instant;

// use rand::prelude::{thread_rng, Rng};
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

fn millis_since(time: &Instant) -> u64 {
    let elapsed = time.elapsed();
    return 1000 * elapsed.as_secs() + elapsed.subsec_millis() as u64;
}

const MAX_DEPTH: usize = 32;

struct Comms {
    file: File
}

impl Comms {
    pub fn new(name: &str) -> Comms {
        Comms {
            file: File::create(name).unwrap()
        }
    }
    fn write(self: &mut Comms, prefix: &str, msg: &str) {
        self.file.write_all(prefix.as_bytes()).unwrap();
        self.file.write_all(msg.as_bytes()).unwrap();
        self.file.write_all(b"\n").unwrap();
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

struct Search<'a> {
    game: &'a mut Game,
    comms: &'a mut Comms,
    nodes: usize,
    start_time: Instant,
    max_millis: u64,
    pv: Vec<Vec<Move>>,
    tmp_pv: Vec<Move>,
    stop_thinking: bool
}

impl<'a> Search<'a> {

    pub fn new(game: &'a mut Game, max_millis: u64, comms: &'a mut Comms) -> Search<'a> {
        let mut pv: Vec<Vec<Move>> = Vec::with_capacity(MAX_DEPTH);
        for _ in 0..MAX_DEPTH {
            pv.push(Vec::with_capacity(MAX_DEPTH));
        }
        Search {
            game,
            comms,
            nodes: 0,
            start_time: Instant::now(),
            max_millis,
            pv,
            tmp_pv: Vec::with_capacity(MAX_DEPTH),
            stop_thinking: false
        }
    }

    pub fn quiesce(self: &mut Search<'a>, alpha: isize, beta: isize,
                   ply: usize, follow_pv: bool) -> isize {
        self.nodes += 1;

        if self.nodes % 1024 == 0 && millis_since(&self.start_time) >= self.max_millis {
            self.stop_thinking = true;
            return 0;  // return value will be ignored
        }

        if ply == MAX_DEPTH - 1 {
            return self.game.evaluate();
        }

        let mut score = self.game.evaluate();
        let mut alpha = alpha;

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }

        let mut moves = self.game.capture_moves();
        let mut follow_pv = follow_pv;

        if follow_pv {
            if ply < self.pv[0].len() {
                if let Some(i) = moves.iter().position(|m| *m == self.pv[0][ply]) {
                    moves.swap(0, i);
                } else {
                    follow_pv = false;
                }
            } else {
                follow_pv = false;
            }
        }

        for mv in &moves {
            let umv = match self.game.make_move(mv) {
                Some(umv) => umv,
                None => continue
            };

            self.pv[ply + 1].clear();    
            score = -self.quiesce(-beta, -alpha, ply + 1, follow_pv);

            self.game.unmake_move(mv, umv);

            if self.stop_thinking {
                return 0;  // return value will be ignored
            }
            if score > alpha {
                if score >= beta {
                    return beta;
                }
                alpha = score;

                self.tmp_pv.push(mv.clone());
                self.tmp_pv.append(&mut self.pv[ply + 1]);
                self.pv[ply].clear();
                self.pv[ply].append(&mut self.tmp_pv);
            }
            follow_pv = false;
        }

        alpha
    }

    pub fn search(self: &mut Search<'a>, alpha: isize, beta: isize,
                  ply: usize, depth: usize, follow_pv: bool) -> isize {
        if ply >= depth {
            return self.quiesce(alpha, beta, ply, follow_pv);
        }

        self.nodes += 1;

        if self.nodes % 1024 == 0 && millis_since(&self.start_time) >= self.max_millis {
            self.stop_thinking = true;
            return 0;  // return value will be ignored
        }

        if ply == MAX_DEPTH - 1 {
            return self.game.evaluate();
        }

        let mut moves = self.game.generate_moves();
        let mut any_legal_moves = false;
        let mut alpha = alpha;
        let mut depth = depth;
        let mut follow_pv = follow_pv;
        let in_check = self.game.in_check();

        if in_check {
            depth += 1;
        }

        if follow_pv {
            if ply < self.pv[0].len() {
                if let Some(i) = moves.iter().position(|m| *m == self.pv[0][ply]) {
                    moves.swap(0, i);
                } else {
                    follow_pv = false;
                }
            } else {
                follow_pv = false;
            }
        }

        for mv in &moves {
            let umv = match self.game.make_move(mv) {
                Some(umv) => umv,
                None => continue
            };
            any_legal_moves = true;

            self.pv[ply + 1].clear();    
            let score = -self.search(-beta, -alpha, ply + 1, depth, follow_pv);

            self.game.unmake_move(mv, umv);

            if self.stop_thinking {
                return 0;  // return value will be ignored
            }
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                self.tmp_pv.push(mv.clone());
                self.tmp_pv.append(&mut self.pv[ply + 1]);
                self.pv[ply].clear();
                self.pv[ply].append(&mut self.tmp_pv);
                if ply == 0 {
                    let millis = millis_since(&self.start_time);
                    let nps = if millis > 0 { 1000 * self.nodes as u64 / millis } else { 0 };
                    let msg = format!("info depth {} score cp {} nodes {} time {} nps {} pv {}",
                        depth, score, self.nodes, millis, nps,
                        self.pv[0].iter().map(|m| m.to_algebraic()).collect::<Vec<String>>().join(" "));
                    self.comms.output(msg);
                }
            }
            follow_pv = false;
        }

        if any_legal_moves {
            alpha
        } else if in_check {
            -100000 + ply as isize
        } else {
            0
        }
    }
}

fn think(game: &mut Game, millis_to_think: u64, search_depth: usize, comms: &mut Comms) -> Option<Move> {
    comms.debug(format!("think movetime {} depth {}", millis_to_think, search_depth));

    let mut search = Search::new(game, millis_to_think, comms);

    for depth in 1..search_depth {
        search.search(-100000, 100000, 0, depth, true);
        if search.stop_thinking {
            break;
        }
    }

    if search.pv[0].len() > 0 {
        Some(search.pv[0][0].clone())
    } else {
        None
    }
}

fn main() {
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0").unwrap();
    let mut comms = Comms::new("comms.txt");

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

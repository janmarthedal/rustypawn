extern crate rand;
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

/*fn think(game: &mut Game) -> Option<Move> {
    let moves = legal_moves(game);
    if moves.is_empty() {
        return None;
    }
    let num = thread_rng().gen_range(0, moves.len());
    Some(moves[num].clone())
}

fn search2(game: &mut Game, depth: usize) -> isize {
    if depth == 0 {
        return game.evaluate();
    }
    let moves = legal_moves(game);
    let mut best_score: Option<isize> = None;
    for mv in &moves {
        let umv = match game.make_move(mv) {
            Some(umv) => umv,
            None => continue
        };
        let score = -search2(game, depth - 1);
        game.unmake_move(mv, umv);
        best_score = match best_score {
            Some(bs) => if score > bs { Some(score) } else { best_score },
            None => Some(score)
        };
    }
    if let Some(score) = best_score {
        score
    } else if game.in_check() {
        -1000000
    } else {
        0
    }
}

fn think2(game: &mut Game, depth: usize) -> Option<Move> {
    let moves = legal_moves(game);
    let mut best_move: Option<(Move, isize)> = None;
    for mv in moves {
        let umv = match game.make_move(&mv) {
            Some(umv) => umv,
            None => continue
        };
        let score = -search2(game, depth - 1);
        game.unmake_move(&mv, umv);
        best_move = match best_move {
            Some((_, bsc)) => if score > bsc { Some((mv, score)) } else { best_move },
            None => Some((mv, score))
        };
    }
    if let Some((mv, _)) = best_move {
        Some(mv)
    } else {
        None
    }
}*/

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
    pub fn debug(self: &mut Comms, msg: &str) {
        self.write("! ", msg);
    }
}

struct Search<'a> {
    game: &'a mut Game,
    comms: &'a mut Comms,
    nodes: usize,
    start_time: Instant,
    max_millis: u64,
    pv: Vec<Vec<Move>>,
    tmp_pv: Vec<Move>
}

impl<'a> Search<'a> {

    pub fn new(game: &'a mut Game, max_millis: u64, comms: &'a mut Comms) -> Search<'a> {
        let mut pv: Vec<Vec<Move>> = Vec::with_capacity(MAX_DEPTH);
        for _ in 1..MAX_DEPTH {
            pv.push(Vec::with_capacity(MAX_DEPTH));
        }
        Search {
            game,
            comms,
            nodes: 0,
            start_time: Instant::now(),
            max_millis,
            pv,
            tmp_pv: Vec::with_capacity(MAX_DEPTH)
        }
    }

    pub fn search(self: &mut Search<'a>, alpha: isize, beta: isize,
                  ply: usize, depth: usize, follow_pv: bool) -> isize {
        self.nodes += 1;
        if ply >= depth {
            self.pv[ply].clear();
            return self.game.evaluate();
        }

        let mut moves = self.game.generate_moves();
        let mut any_legal_moves = false;
        let mut alpha = alpha;
        let mut new_depth = depth;
        let mut follow_pv = follow_pv;
        let in_check = self.game.in_check();

        if in_check {
            new_depth += 1;
        }

        if follow_pv {
            if ply < self.pv[0].len() {
                if let Some(i) = moves.iter().position(|m| *m == self.pv[0][ply]) {
                    self.comms.debug("Swapping");
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

            let score = -self.search(-beta, -alpha, ply + 1, new_depth, follow_pv);
            follow_pv = false;

            self.game.unmake_move(mv, umv);

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
                    let nps = if millis > 0 { self.nodes as u64 / millis / 1000 } else { 0 };
                    let msg = format!("info depth {} score cp {} nodes {} time {} nps {} pv {}",
                        depth, score, self.nodes, millis, nps,
                        self.pv[0].iter().map(|m| m.to_algebraic()).collect::<Vec<String>>().join(" "));
                    self.comms.output(msg);
                }
            }
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

fn think3(game: &mut Game, comms: &mut Comms) -> Option<Move> {
    let mut search = Search::new(game, 1000, comms);
    let depth = 6;

    for d in 1..depth {
        search.search(-100000, 100000, 0, d, true);
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
                let args: Vec<&str> = line.split_whitespace().collect();
                match args[0] {
                    "uci" => {
                        comms.output("id name rustypawn");
                        comms.output("id author Jan Marthedal Rasmussen");
                        comms.output("uciok");
                    },
                    "isready" => {
                        comms.output("readyok");
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
                        let mv = match think3(&mut game, &mut comms) {
                            Some(m) => m,
                            None => panic!("No legal move")
                        };
                        comms.output(format!("bestmove {}", mv.to_algebraic()));
                    }
                    _ => continue
                }
            }
            Err(error) => panic!("error: {}", error),
        }
    }
}

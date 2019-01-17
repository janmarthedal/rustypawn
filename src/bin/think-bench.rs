use std::time::Instant;
extern crate rustypawn;

use rustypawn::Game;
use rustypawn::MoveTrait;
use rustypawn::ThinkInfo;
use rustypawn::think;
use rustypawn::millis_since;

struct Comms {}

impl ThinkInfo for Comms {
    fn think_info(self: &mut Comms, depth: usize, score: isize, mate_in: isize, node_count: usize, millis: u64, moves: &Vec<String>) {
        let nps = if millis > 0 { 1000 * node_count as u64 / millis } else { 0 };
        let mate = if mate_in != 0 { format!(" mate {}", mate_in) } else { String::new() };
        println!("info depth {} score cp {}{} nodes {} time {} nps {} pv {}",
            depth, score, mate, node_count, millis, nps, moves.join(" "));
    }
}

fn main() {
    let fen = "1rb2rk1/p4ppp/1p1qp1n1/3n2N1/2pP4/2P3P1/PPQ2PBP/R1B1R1K1 w - - 4 17";
    let mut game = Game::from_fen(fen).unwrap();
    let mut comms = Comms {};

    let start = Instant::now();

    let mv = match think(&mut game, 1 << 20, 6, &mut comms) {
        Some(m) => m,
        None => panic!("No legal move")
    };

    println!("bestmove {}", mv.to_algebraic());
    println!("Time: {} ms", millis_since(&start));
}

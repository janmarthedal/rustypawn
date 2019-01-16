use std::time::Instant;
extern crate rustypawn;

use rustypawn::Game;

fn perft_sub(game: &mut Game, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let move_list = game.generate_moves();

    let mut result = 0;
    for mv in &move_list {
        if game.make_move(&mv) {
            let count_sub = perft_sub(game, depth - 1);
            game.unmake_move(&mv);
            result += count_sub;
        }
    }

    result
}

pub fn perft(fen: &str, depth: usize) -> usize {
    match Game::from_fen(fen) {
        Ok(mut game) => perft_sub(&mut game, depth),
        Err(_) => 0
    }
}

#[cfg(test)]
mod tests {
    use super::perft;

    #[test]
    fn perft_initial_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0";
        assert_eq!(perft(fen, 1), 20);
        assert_eq!(perft(fen, 2), 400);
        assert_eq!(perft(fen, 3), 8902);
        assert_eq!(perft(fen, 4), 197281);
    }

    #[test]
    fn perft_position_2() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0";
        assert_eq!(perft(fen, 1), 48);
        assert_eq!(perft(fen, 2), 2039);
        assert_eq!(perft(fen, 3), 97862);
    }

    #[test]
    fn perft_position_3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0";
        assert_eq!(perft(fen, 1), 14);
        assert_eq!(perft(fen, 2), 191);
        assert_eq!(perft(fen, 3), 2812);
        assert_eq!(perft(fen, 4), 43238);
        assert_eq!(perft(fen, 5), 674624);
    }

    #[test]
    fn perft_position_4w() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0";
        assert_eq!(perft(fen, 1), 6);
        assert_eq!(perft(fen, 2), 264);
        assert_eq!(perft(fen, 3), 9467);
        assert_eq!(perft(fen, 4), 422333);
    }

    #[test]
    fn perft_position_4b() {
        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0";
        assert_eq!(perft(fen, 1), 6);
        assert_eq!(perft(fen, 2), 264);
        assert_eq!(perft(fen, 3), 9467);
        assert_eq!(perft(fen, 4), 422333);
    }

    #[test]
    fn perft_position_5() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        assert_eq!(perft(fen, 1), 44);
        assert_eq!(perft(fen, 2), 1486);
        assert_eq!(perft(fen, 3), 62379);
    }

    #[test]
    fn perft_position_6() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        assert_eq!(perft(fen, 1), 46);
        assert_eq!(perft(fen, 2), 2079);
        assert_eq!(perft(fen, 3), 89890);
    }

}

fn run_perft(name: &str, fen: &str, depth: usize, verification: usize) {
    let count = perft(fen, depth);
    assert_eq!(count, verification);
    println!("{} {}", name, count);
}


fn main() {
    let now = Instant::now();

    run_perft("Initial position", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0", 5, 4865609);
    run_perft("Kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -", 4, 4085603);
    run_perft("Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 6, 11030083);
    run_perft("Position 6", "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 4, 3894594);

    let elapsed = now.elapsed();
    println!("Time: {} ms", 1000 * elapsed.as_secs() + elapsed.subsec_millis() as u64);
}

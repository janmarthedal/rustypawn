use std::io;

use rustypawn::Game;

fn main() {
    let mut input = String::new();
    let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0");
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let cmd = input.trim();
                println!("{}", cmd);
            }
            Err(error) => panic!("error: {}", error),
        }
    }
}

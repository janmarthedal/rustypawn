use std::io;

use rustypawn::init_game;

fn main() {
    let mut input = String::new();
    let mut game = init_game("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0");
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

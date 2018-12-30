const EMPTY: u8 = 0;
const PAWN: u8 = 1;
const BISHOP: u8 = 2;
const KNIGHT: u8 = 3;
const ROOK: u8 = 4;
const QUEEN: u8 = 5;
const KING: u8 = 6;
const WHITE: u8 = 64;
const BLACK: u8 = 128;
const OFF_BOARD: u8 = WHITE | BLACK;
const COLOR_MASK: u8 = WHITE | BLACK;
const PIECE_MASK: u8 = 7;

struct Game {
    board: [u8; 120],
    king_white: u8,
    king_black: u8
}

const MAILBOX: [usize; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98
];

fn new_game() -> Game {
    let mut game = Game {
        board: [OFF_BOARD; 120],
        king_white: 0,
        king_black: 0
    };
    for i in 0..64 {
        game.board[MAILBOX[i] as usize] = EMPTY;
    }
    game
}

fn init_game(game: &mut Game) {
    game.board[21] = ROOK | BLACK;
    game.board[22] = KNIGHT | BLACK;
    game.board[23] = BISHOP | BLACK;
    game.board[24] = QUEEN | BLACK;
    game.board[25] = KING | BLACK;
    game.board[26] = BISHOP | BLACK;
    game.board[27] = KNIGHT | BLACK;
    game.board[28] = ROOK | BLACK;
    // game.board[31] = PAWN | BLACK;
    // game.board[32] = PAWN | BLACK;
    // game.board[33] = PAWN | BLACK;
    // game.board[34] = PAWN | BLACK;
    // game.board[35] = PAWN | BLACK;
    // game.board[36] = PAWN | BLACK;
    // game.board[37] = PAWN | BLACK;
    // game.board[38] = PAWN | BLACK;
    // game.board[81] = PAWN | WHITE;
    // game.board[82] = PAWN | WHITE;
    // game.board[83] = PAWN | WHITE;
    // game.board[84] = PAWN | WHITE;
    // game.board[85] = PAWN | WHITE;
    // game.board[86] = PAWN | WHITE;
    // game.board[87] = PAWN | WHITE;
    // game.board[88] = PAWN | WHITE;
    // game.board[91] = ROOK | WHITE;
    game.board[92] = KNIGHT | WHITE;
    game.board[93] = BISHOP | WHITE;
    game.board[94] = QUEEN | WHITE;
    game.board[95] = KING | WHITE;
    game.board[96] = BISHOP | WHITE;
    game.board[97] = KNIGHT | WHITE;
    game.board[98] = ROOK | WHITE;
    game.king_black = 25;
    game.king_white = 95;
}

const ROOK_MOVEMENTS: [isize; 4] = [-1, 1, -10, 10];

fn generate_moves(game: &Game) {
    let side = WHITE;
    let xside = if side == WHITE { BLACK } else { WHITE};
    let board = game.board;
    println!("white pawn value {}", WHITE | PAWN);
    for i in 0..64 {
        let from = MAILBOX[i];
        let piece = board[from];
        if piece & COLOR_MASK == side {
            let base_piece = piece & PIECE_MASK;
            if base_piece == PAWN {
                println!("pawn");
            } else if base_piece == ROOK {
                for delta in ROOK_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while board[to] == EMPTY {
                        println!("ROOK from {} to {}", from, to);
                        to = ((to as isize) + delta) as usize;
                    }
                    if board[to] & COLOR_MASK == xside {
                        println!("ROOK from {} to {}", from, to);
                    }
                }
            }
        }
    }
}

fn main() {
    println!("PAWN: {}", PAWN);
    println!("BISHOP: {}", BISHOP);
    println!("KNIGHT: {}", KNIGHT);
    println!("ROOK: {}", ROOK);
    println!("QUEEN: {}", QUEEN);
    println!("KING: {}", KING);

    let mut game = new_game();
    init_game(&mut game);

    println!("white king pos: {}", game.king_white);
    println!("black king pos: {}", game.king_black);

    generate_moves(&game);
}

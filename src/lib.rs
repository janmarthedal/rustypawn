use rand::prelude::*;
use std::time::Instant;

const EMPTY: usize = 0;
const PAWN: usize = 1;
const BISHOP: usize = 2;
const KNIGHT: usize = 3;
const ROOK: usize = 4;
const QUEEN: usize = 5;
const KING: usize = 6;
const WHITE: usize = 64;
const BLACK: usize = 128;
const OFF_BOARD: usize = WHITE | BLACK;
const COLOR_MASK: usize = WHITE | BLACK;
const PIECE_MASK: usize = 7;
const CASTLING_KING_WHITE: usize = 1;
const CASTLING_QUEEN_WHITE: usize = 2;
const CASTLING_KING_BLACK: usize = 4;
const CASTLING_QUEEN_BLACK: usize = 8;
const WHITE_PAWN: usize = WHITE | PAWN;
const WHITE_BISHOP: usize = WHITE | BISHOP;
const WHITE_KNIGHT: usize = WHITE | KNIGHT;
const WHITE_ROOK: usize = WHITE | ROOK;
const WHITE_QUEEN: usize = WHITE | QUEEN;
// const WHITE_KING: usize = WHITE | KING;
const BLACK_PAWN: usize = BLACK | PAWN;
const BLACK_BISHOP: usize = BLACK | BISHOP;
const BLACK_KNIGHT: usize = BLACK | KNIGHT;
const BLACK_ROOK: usize = BLACK | ROOK;
const BLACK_QUEEN: usize = BLACK | QUEEN;
// const BLACK_KING: usize = BLACK | KING;

const CASTLE_MASK: [usize; 120] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0, 0,
    0,  0,  0,  0,  0,  0,  0,  0,  0, 0,
    0,  7, 15, 15, 15,  3, 15, 15, 11, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 15, 15, 15, 15, 15, 15, 15, 15, 0,
    0, 13, 15, 15, 15, 12, 15, 15, 14, 0,
    0,  0,  0,  0,  0,  0,  0,  0,  0, 0,
    0,  0,  0,  0,  0,  0,  0,  0,  0, 0,
];

fn pos_to_algebraic(pos: usize) -> String {
    format!("{}{}", (96 + (pos % 10)) as u8 as char, (58 - (pos / 10)) as u8 as char)
}

pub type Move = u64;   // promoted << 16 | to << 8 | from

const DUMMY_MOVE: Move = 0;  // non-existent move

pub trait MoveTrait {
    fn new_basic(from: usize, to: usize) -> Move;
    fn new_promotion(from: usize, to: usize, promotion: usize) -> Move;
    fn from(self) -> usize;
    fn to(self) -> usize;
    fn promotion(self) -> usize;
    fn to_algebraic(self) -> String;
}

impl MoveTrait for Move {
    fn new_basic(from: usize, to: usize) -> Move {
        (to << 8 | from) as Move
    }
    fn new_promotion(from: usize, to: usize, promotion: usize) -> Move {
        (promotion << 16 | to << 8 | from) as Move
    }
    fn from(self) -> usize {
        (self & 0xff) as usize
    }
    fn to(self) -> usize {
        ((self >> 8) & 0xff) as usize
    }
    fn promotion(self) -> usize {
        ((self >> 16) & 0xff) as usize
    }
    fn to_algebraic(self) -> String {
        let promotion = match self.promotion() {
            EMPTY => "",
            BISHOP => "b",
            KNIGHT => "n",
            ROOK => "r",
            QUEEN => "q",
            _ => panic!("to_algebraic")
        };
        format!("{}{}{}", pos_to_algebraic(self.from()), pos_to_algebraic(self.to()), promotion)
    }
}

type State = usize;

trait StateTrait {
    fn draw_ply(self) -> usize;
    fn ep(self) -> usize;
}

impl StateTrait for State {
    fn draw_ply(self) -> usize {
        (self >> 24) & 127
    }
    fn ep(self) -> usize {
        (self >> 16) & 127
    }
}

struct HistoryItem {
    unmove: u64,  // captured << 32 | state
    hash: u64,
}

pub struct Game {
    board: [usize; 120],
    state: State,  // draw_ply << 24 | ep << 16 | castling << 8 | turn
    king_white: usize,
    king_black: usize,
    piece_hashes: [u64; 12 * 64],
    black_hash: u64,
    castling_hashes: [u64; 16],
    ep_hashes: [u64; 8],
    hash: u64,
    history: Vec<HistoryItem>,
}

const REV8X8: [usize; 120] = [
    99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99,  0,  1,  2,  3,  4,  5,  6,  7, 99,
	99,  8,  9, 10, 11, 12, 13, 14, 15, 99,
	99, 16, 17, 18, 19, 20, 21, 22, 23, 99,
	99, 24, 25, 26, 27, 28, 29, 30, 31, 99,
	99, 32, 33, 34, 35, 36, 37, 38, 39, 99,
	99, 40, 41, 42, 43, 44, 45, 46, 47, 99,
	99, 48, 49, 50, 51, 52, 53, 54, 55, 99,
	99, 56, 57, 58, 59, 60, 61, 62, 63, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99
];

const MAP8X8: [usize; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28,
    31, 32, 33, 34, 35, 36, 37, 38,
    41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58,
    61, 62, 63, 64, 65, 66, 67, 68,
    71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88,
    91, 92, 93, 94, 95, 96, 97, 98
];

const PIECE_ASCII: &str = " PBNRQKpbnrqk";
const PIECE_VALUES: [usize; 13] = [
    EMPTY,
    WHITE | PAWN, WHITE | BISHOP, WHITE | KNIGHT, WHITE | ROOK, WHITE | QUEEN, WHITE | KING,
    BLACK | PAWN, BLACK | BISHOP, BLACK | KNIGHT, BLACK | ROOK, BLACK | QUEEN, BLACK | KING
];
const BISHOP_MOVEMENTS: [isize; 4] = [-11, -9, 9, 11];
const KNIGHT_MOVEMENTS: [isize; 8] = [-21, -19, -12, -8, 8, 12, 19, 21];
const ROOK_MOVEMENTS: [isize; 4] = [-1, 1, -10, 10];
const KING_MOVEMENTS: [isize; 8] = [-1, 1, -10, 10, -11, -9, 9, 11];
const PAWN_VALUE: isize = 100;
const BISHOP_VALUE: isize = 300;
const KNIGHT_VALUE: isize = 300;
const ROOK_VALUE: isize = 500;
const QUEEN_VALUE: isize = 900;
pub const MAX_DEPTH: usize = 32;
const MATE_VALUE: isize = 100000;

const PAWN_PCSQ: [isize; 64] = [
	  0,   0,   0,   0,   0,   0,   0,   0,
	  5,  10,  15,  20,  20,  15,  10,   5,
	  4,   8,  12,  16,  16,  12,   8,   4,
	  3,   6,   9,  12,  12,   9,   6,   3,
	  2,   4,   6,   8,   8,   6,   4,   2,
	  1,   2,   3, -10, -10,   3,   2,   1,
	  0,   0,   0, -40, -40,   0,   0,   0,
	  0,   0,   0,   0,   0,   0,   0,   0
];

const BISHOP_PCSQ: [isize; 64] = [
	-10, -10, -10, -10, -10, -10, -10, -10,
	-10,   0,   0,   0,   0,   0,   0, -10,
	-10,   0,   5,   5,   5,   5,   0, -10,
	-10,   0,   5,  10,  10,   5,   0, -10,
	-10,   0,   5,  10,  10,   5,   0, -10,
	-10,   0,   5,   5,   5,   5,   0, -10,
	-10,   0,   0,   0,   0,   0,   0, -10,
	-10, -10, -20, -10, -10, -20, -10, -10
];

const KNIGHT_PCSQ: [isize; 64] = [
	-10, -10, -10, -10, -10, -10, -10, -10,
	-10,   0,   0,   0,   0,   0,   0, -10,
	-10,   0,   5,   5,   5,   5,   0, -10,
	-10,   0,   5,  10,  10,   5,   0, -10,
	-10,   0,   5,  10,  10,   5,   0, -10,
	-10,   0,   5,   5,   5,   5,   0, -10,
	-10,   0,   0,   0,   0,   0,   0, -10,
	-10, -30, -10, -10, -10, -10, -30, -10
];

const FLIP: [usize; 64] = [
	 56,  57,  58,  59,  60,  61,  62,  63,
	 48,  49,  50,  51,  52,  53,  54,  55,
	 40,  41,  42,  43,  44,  45,  46,  47,
	 32,  33,  34,  35,  36,  37,  38,  39,
	 24,  25,  26,  27,  28,  29,  30,  31,
	 16,  17,  18,  19,  20,  21,  22,  23,
	  8,   9,  10,  11,  12,  13,  14,  15,
	  0,   1,   2,   3,   4,   5,   6,   7
];

pub fn algebraic_to_pos(s: &str) -> Option<usize> {
    let mut iter = s.chars();
    let c = match iter.next() {
        Option::Some(c) => {
            match c.to_digit(18) {
                Option::Some(v) => {
                    if v < 10 {
                        return Option::None
                    }
                    (v - 10) as usize
                },
                Option::None => return Option::None
            }
        },
        Option::None => return Option::None
    };
    let r = match iter.next() {
        Option::Some(c) => {
            match c.to_digit(10) {
                Option::Some(v) => (8 - v) as usize,
                Option::None => return Option::None
            }
        },
        Option::None => return Option::None
    };
    match iter.next() {
        Option::Some(_) => Option::None,
        Option::None => Option::Some(8 * r + c)
    }
}

pub fn algebraic_to_move(s: &str) -> Move {
    Move::new_promotion(
        MAP8X8[algebraic_to_pos(&s[0..2]).unwrap()],
        MAP8X8[algebraic_to_pos(&s[2..4]).unwrap()],
        match &s[4..] {
            "b" => BISHOP,
            "n" => KNIGHT,
            "r" => ROOK,
            "q" => QUEEN,
            "" => EMPTY,
            _ => panic!("algebraic_to_move: illegal promotion")
        }
    )
}

fn add_move(move_list: &mut Vec<Move>, from: usize, to: usize) {
    move_list.push(Move::new_basic(from, to));
}

fn add_promotion(move_list: &mut Vec<Move>, from: usize, to: usize) {
    move_list.push(Move::new_promotion(from, to, BISHOP));
    move_list.push(Move::new_promotion(from, to, KNIGHT));
    move_list.push(Move::new_promotion(from, to, ROOK));
    move_list.push(Move::new_promotion(from, to, QUEEN));
}

const DOUBLED_PAWN_PENALTY: isize = 10;
const ISOLATED_PAWN_PENALTY: isize = 20;
const BACKWARDS_PAWN_PENALTY: isize = 8;
const PASSED_PAWN_BONUS: isize = 20;
const ROOK_SEMI_OPEN_FILE_BONUS: isize = 10;
const ROOK_OPEN_FILE_BONUS: isize = 15;
const ROOK_ON_SEVENTH_BONUS: isize = 20;

fn evaluate_white_pawn(i: usize, white_pawn_rank: &[usize; 10], black_pawn_rank: &[usize; 10]) -> isize {
    let f = i % 8 + 1;
    let r = i / 8;
    let mut s = PAWN_PCSQ[i];

    if white_pawn_rank[f] > r {
        s -= DOUBLED_PAWN_PENALTY;
    }

    if white_pawn_rank[f - 1] == 0 && white_pawn_rank[f + 1] == 0 {
        s -= ISOLATED_PAWN_PENALTY;
    } else if white_pawn_rank[f - 1] < r && white_pawn_rank[f + 1] < r {
        s -= BACKWARDS_PAWN_PENALTY;
    }

    if black_pawn_rank[f - 1] >= r && black_pawn_rank[f] >= r && black_pawn_rank[f + 1] >= r {
        s += (7 - r as isize) * PASSED_PAWN_BONUS;
    }

    s
}

fn evaluate_black_pawn(i: usize, white_pawn_rank: &[usize; 10], black_pawn_rank: &[usize; 10]) -> isize {
    let f = i % 8 + 1;
    let r = i / 8;
    let mut s = PAWN_PCSQ[FLIP[i]];

    if black_pawn_rank[f] < r {
        s -= DOUBLED_PAWN_PENALTY;
    }

    if black_pawn_rank[f - 1] == 7 && black_pawn_rank[f + 1] == 7 {
        s -= ISOLATED_PAWN_PENALTY;
    } else if black_pawn_rank[f - 1] > r && black_pawn_rank[f + 1] > r {
        s -= BACKWARDS_PAWN_PENALTY;
    }

    if white_pawn_rank[f - 1] <= r && white_pawn_rank[f] <= r && white_pawn_rank[f + 1] <= r {
        s += r as isize * PASSED_PAWN_BONUS;
    }

    s
}

impl Game {

    fn new() -> Game {
        let mut rng = rand::thread_rng();
        Game {
            board: {
                let mut b: [usize; 120] = [OFF_BOARD; 120];
                for i in 0..64 { b[MAP8X8[i]] = EMPTY; }
                b
            },
            state: 0,
            king_white: 0,
            king_black: 0,
            piece_hashes: {
                let mut h: [u64; 12 * 64] = [0; 12 * 64];
                for k in 0..(12 * 64) { h[k] = rng.next_u64(); }
                h
            },
            black_hash: rng.next_u64(),
            castling_hashes: {
                let mut h: [u64; 16] = [0; 16];
                for k in 0..16 { h[k] = rng.next_u64(); }
                h
            },
            ep_hashes: {
                let mut h: [u64; 8] = [0; 8];
                for k in 0..8 { h[k] = rng.next_u64(); }
                h
            },
            hash: 0,
            history: Vec::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Result<Game, &str> {
        let mut iter = fen.split_whitespace();
        let mut game = Game::new();
        match iter.next() {
            Some(s) => {
                let mut pos: usize = 0;
                for c in s.chars() {
                    if c == '/' {
                        continue;
                    }
                    match c.to_digit(10) {
                        Option::Some(n) => {
                            pos += n as usize;
                            continue;
                        },
                        Option::None => {}
                    };
                    match PIECE_ASCII.find(c) {
                        Option::Some(idx) => {
                            game.board[MAP8X8[pos]] = PIECE_VALUES[idx];
                            pos += 1;
                        },
                        Option::None => return Result::Err("Illegal FEN character")
                    };
                }
            },
            None => return Result::Err("Empty FEN")
        };
        let side = match iter.next() {
            Some(s) => match &s.to_lowercase()[..] {
                "w" => WHITE,
                "b" => BLACK,
                _ => return Result::Err("Illegal side string")
            },
            None => return Result::Err("Missing side")
        };
        let mut castling: usize = 0;
        match iter.next() {
            Some(s) => {
                for c in s.chars() {
                    match c {
                        'K' => castling = castling | CASTLING_KING_WHITE,
                        'Q' => castling = castling | CASTLING_QUEEN_WHITE,
                        'k' => castling = castling | CASTLING_KING_BLACK,
                        'q' => castling = castling | CASTLING_QUEEN_BLACK,
                        '-' => continue,
                        _ => return Result::Err("Illegal castling character")
                    }
                }
            },
            None => {}
        };
        let ep = match iter.next() {
            Some(s) => match s {
                "-" => 0,
                _ => match algebraic_to_pos(s) {
                    Some(p) => MAP8X8[p],
                    None => return Result::Err("Illegal en passant string")
                }
            },
            None => 0
        };
        let draw_ply = match iter.next() {
            Some(s) => match s.parse::<usize>() {
                Ok(v) => {
                    if v >= 100 {
                        return Result::Err("Illegal ply value"); 
                    }
                    v
                },
                Err(_) => return Result::Err("Illegal ply format")
            },
            None => 0
        };
        game.state = draw_ply << 24 | ep << 16 | castling << 8 | side;
        game.king_white = match game.board.iter().position(|&p| p == WHITE | KING) {
            Some(i) => i,
            None => return Result::Err("No white king")
        };
        game.king_black = match game.board.iter().position(|&p| p == BLACK | KING) {
            Some(i) => i,
            None => return Result::Err("No black king")
        };
        game.set_hash();
        return Result::Ok(game);
    }

    fn set_hash(self: &mut Game) {
        let mut hash: u64 = 0;
        for i in 0..64 {
            let piece = self.board[MAP8X8[i]];
            if piece != EMPTY {
                let mut n = (piece & PIECE_MASK) - 1;
                if (piece & COLOR_MASK) == BLACK {
                    n += 6;
                }
                hash ^= self.piece_hashes[n * 64 + i];
            }
        }
        if !self.white_to_move() {
            hash ^= self.black_hash;
        }
        hash ^= self.castling_hashes[(self.state >> 8) & 15];
        let ep = self.state.ep();
        if ep != 0 {
            hash ^= self.ep_hashes[(ep % 10) - 1];
        }
        self.hash = hash;
    }

    fn is_attacked_by(self: &Game, pos: usize, color: usize) -> bool {
        if color == WHITE {
            if self.board[pos + 9] == (PAWN | WHITE) || self.board[pos + 11] == (PAWN | WHITE) {
                return true;
            }
        } else {
            if self.board[pos - 9] == (PAWN | BLACK) || self.board[pos - 11] == (PAWN | BLACK) {
                return true;
            }
        }
        for delta in BISHOP_MOVEMENTS.iter() {
            let mut to = ((pos as isize) + delta) as usize;
            while self.board[to] == EMPTY {
                to = ((to as isize) + delta) as usize;
            }
            if self.board[to] == BISHOP | color {
                return true;
            }
        }
        for delta in KNIGHT_MOVEMENTS.iter() {
            if self.board[((pos as isize) + delta) as usize] == KNIGHT | color {
                return true;
            }
        }
        for delta in ROOK_MOVEMENTS.iter() {
            let mut to = ((pos as isize) + delta) as usize;
            while self.board[to] == EMPTY {
                to = ((to as isize) + delta) as usize;
            }
            if self.board[to] == ROOK | color {
                return true;
            }
        }
        for delta in KING_MOVEMENTS.iter() {
            let mut to = ((pos as isize) + delta) as usize;
            while self.board[to] == EMPTY {
                to = ((to as isize) + delta) as usize;
            }
            if self.board[to] == QUEEN | color {
                return true;
            }
        }
        for delta in KING_MOVEMENTS.iter() {
            if self.board[((pos as isize) + delta) as usize] == KING | color {
                return true;
            }
        }
        return false;
    }

    pub fn generate_moves(self: &Game) -> Vec<Move> {
        let side = self.state & 0xff;
        let xside = if side == WHITE { BLACK } else { WHITE };
        let castling = (self.state >> 8) & 0xff;
        let ep = self.state.ep();

        let mut move_list = Vec::with_capacity(218);
        for i in 0..64 {
            let from = MAP8X8[i];
            let piece = self.board[from];
            if piece & COLOR_MASK == side {
                match piece & PIECE_MASK {
                    PAWN if side == WHITE => {
                        if i >> 3 == 1 {
                            if self.board[from - 10] == EMPTY {
                                add_promotion(&mut move_list, from, from - 10);
                            }
                            if self.board[from - 11] & COLOR_MASK == BLACK {
                                add_promotion(&mut move_list, from, from - 11);
                            }
                            if self.board[from - 9] & COLOR_MASK == BLACK {
                                add_promotion(&mut move_list, from, from - 9);
                            }
                        } else {
                            if self.board[from - 10] == EMPTY {
                                add_move(&mut move_list, from, from - 10);
                                if i >> 3 == 6 && self.board[from - 20] == EMPTY {
                                    add_move(&mut move_list, from, from - 20);
                                }
                            }
                            if self.board[from - 11] & COLOR_MASK == BLACK || from - 11 == ep {
                                add_move(&mut move_list, from, from - 11);
                            }
                            if self.board[from - 9] & COLOR_MASK == BLACK || from - 9 == ep {
                                add_move(&mut move_list, from, from - 9);
                            }
                        }
                    },
                    PAWN if side == BLACK => {
                        if i >> 3 == 6 {
                            if self.board[from + 10] == EMPTY {
                                add_promotion(&mut move_list, from, from + 10);
                            }
                            if self.board[from + 11] & COLOR_MASK == WHITE {
                                add_promotion(&mut move_list, from, from + 11);
                            }
                            if self.board[from + 9] & COLOR_MASK == WHITE {
                                add_promotion(&mut move_list, from, from + 9);
                            }
                        } else {
                            if self.board[from + 10] == EMPTY {
                                add_move(&mut move_list, from, from + 10);
                                if i >> 3 == 1 && self.board[from + 20] == EMPTY {
                                    add_move(&mut move_list, from, from + 20);
                                }
                            }
                            if self.board[from + 11] & COLOR_MASK == WHITE || from + 11 == ep {
                                add_move(&mut move_list, from, from + 11);
                            }
                            if self.board[from + 9] & COLOR_MASK == WHITE || from + 9 == ep {
                                add_move(&mut move_list, from, from + 9);
                            }
                        }
                    },
                    BISHOP => {
                        for delta in BISHOP_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                add_move(&mut move_list, from, to);
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    KNIGHT => {
                        for delta in KNIGHT_MOVEMENTS.iter() {
                            let to = ((from as isize) + delta) as usize;
                            if self.board[to] == EMPTY || self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    ROOK => {
                        for delta in ROOK_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                add_move(&mut move_list, from, to);
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    QUEEN => {
                        for delta in KING_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                add_move(&mut move_list, from, to);
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    KING => {
                        if from == 95 && (castling & (CASTLING_QUEEN_WHITE | CASTLING_KING_WHITE)) != 0
                                && !self.is_attacked_by(95, BLACK) {
                            if (castling & CASTLING_QUEEN_WHITE) != 0
                                    && self.board[94] == EMPTY && self.board[93] == EMPTY && self.board[92] == EMPTY
                                    && !self.is_attacked_by(94, BLACK) {
                                add_move(&mut move_list, 95, 93);
                            }
                            if (castling & CASTLING_KING_WHITE) != 0
                                    && self.board[96] == EMPTY && self.board[97] == EMPTY
                                    && !self.is_attacked_by(96, BLACK) {
                                add_move(&mut move_list, 95, 97);
                            }
                        } else if from == 25 && (castling & (CASTLING_QUEEN_BLACK | CASTLING_KING_BLACK)) != 0
                                && !self.is_attacked_by(25, WHITE) {
                            if (castling & CASTLING_QUEEN_BLACK) != 0
                                    && self.board[24] == EMPTY && self.board[23] == EMPTY && self.board[22] == EMPTY
                                    && !self.is_attacked_by(24, WHITE) {
                                add_move(&mut move_list, 25, 23);
                            }
                            if (castling & CASTLING_KING_BLACK) != 0
                                    && self.board[26] == EMPTY && self.board[27] == EMPTY
                                    && !self.is_attacked_by(26, WHITE) {
                                add_move(&mut move_list, 25, 27);
                            }
                        }
                        for delta in KING_MOVEMENTS.iter() {
                            let to = ((from as isize) + delta) as usize;
                            if self.board[to] == EMPTY || self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    _ => panic!("generate_moves: unknown piece")
                }
            }
        }
        move_list
    }

    pub fn capture_moves(self: &Game) -> Vec<Move> {
        let side = self.state & 0xff;
        let xside = if side == WHITE { BLACK } else { WHITE };
        let ep = self.state.ep();

        let mut move_list = Vec::with_capacity(218);
        for i in 0..64 {
            let from = MAP8X8[i];
            let piece = self.board[from];
            if piece & COLOR_MASK == side {
                match piece & PIECE_MASK {
                    PAWN if side == WHITE => {
                        if i >> 3 == 1 {
                            if self.board[from - 11] & COLOR_MASK == BLACK {
                                add_promotion(&mut move_list, from, from - 11);
                            }
                            if self.board[from - 9] & COLOR_MASK == BLACK {
                                add_promotion(&mut move_list, from, from - 9);
                            }
                        } else {
                            if self.board[from - 11] & COLOR_MASK == BLACK || from - 11 == ep {
                                add_move(&mut move_list, from, from - 11);
                            }
                            if self.board[from - 9] & COLOR_MASK == BLACK || from - 9 == ep {
                                add_move(&mut move_list, from, from - 9);
                            }
                        }
                    },
                    PAWN if side == BLACK => {
                        if i >> 3 == 6 {
                            if self.board[from + 11] & COLOR_MASK == WHITE {
                                add_promotion(&mut move_list, from, from + 11);
                            }
                            if self.board[from + 9] & COLOR_MASK == WHITE {
                                add_promotion(&mut move_list, from, from + 9);
                            }
                        } else {
                            if self.board[from + 11] & COLOR_MASK == WHITE || from + 11 == ep {
                                add_move(&mut move_list, from, from + 11);
                            }
                            if self.board[from + 9] & COLOR_MASK == WHITE || from + 9 == ep {
                                add_move(&mut move_list, from, from + 9);
                            }
                        }
                    },
                    BISHOP => {
                        for delta in BISHOP_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    KNIGHT => {
                        for delta in KNIGHT_MOVEMENTS.iter() {
                            let to = ((from as isize) + delta) as usize;
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    ROOK => {
                        for delta in ROOK_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    QUEEN => {
                        for delta in KING_MOVEMENTS.iter() {
                            let mut to = ((from as isize) + delta) as usize;
                            while self.board[to] == EMPTY {
                                to = ((to as isize) + delta) as usize;
                            }
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    KING => {
                        for delta in KING_MOVEMENTS.iter() {
                            let to = ((from as isize) + delta) as usize;
                            if self.board[to] & COLOR_MASK == xside {
                                add_move(&mut move_list, from, to);
                            }
                        }
                    },
                    _ => panic!("capture_moves: unknown piece")
                }
            }
        }
        move_list
    }

    pub fn make_move(self: &mut Game, mv: Move) -> bool {
        let from = mv.from();
        let to = mv.to();
        let promoted = mv.promotion();
        let piece = self.board[from];
        let captured = self.board[to];
        let from_state = self.state;
        let from_ep = from_state.ep();
        let mut to_ep = 0;
        let from_draw_ply = from_state.draw_ply();
        let mut to_draw_ply = if captured == EMPTY { from_draw_ply + 1 } else { 0 };
        let from_castling = (from_state >> 8) & 0xff;
        let to_castling = from_castling & CASTLE_MASK[from] & CASTLE_MASK[to];
        let side = from_state & 0xff;
        let xside = if side == WHITE { BLACK } else { WHITE };

        self.board[to] = if promoted != EMPTY { promoted | side } else { piece };
        self.board[from] = EMPTY;

        if piece == PAWN | WHITE {
            if to == from_ep {
                self.board[to + 10] = EMPTY;
            } else if to == from - 20 {
                to_ep = from - 10;
            }
            to_draw_ply = 0;
        } else if piece == PAWN | BLACK {
            if to == from_ep {
                self.board[to - 10] = EMPTY;
            } else if to == from + 20 {
                to_ep = from + 10;
            }
            to_draw_ply = 0;
        } else if piece == KING | WHITE {
            self.king_white = to;
            if from == 95 {
                if to == 93 {
                    self.board[91] = EMPTY;
                    self.board[94] = ROOK | WHITE;
                } else if to == 97 {
                    self.board[98] = EMPTY;
                    self.board[96] = ROOK | WHITE;
                }
            }
        } else if piece == KING | BLACK {
            self.king_black = to;
            if from == 25 {
                if to == 23 {
                    self.board[21] = EMPTY;
                    self.board[24] = ROOK | BLACK;
                } else if to == 27 {
                    self.board[28] = EMPTY;
                    self.board[26] = ROOK | BLACK;
                }
            }
        }

        self.state = to_draw_ply << 24 | to_ep << 16 | to_castling << 8 | xside;
        self.history.push(HistoryItem {
            unmove: (captured as u64) << 32 | from_state as u64,
            hash: self.hash
        });

        if self.is_attacked_by(if side == WHITE { self.king_white } else { self.king_black }, xside) {
            self.unmake_move(mv);
            return false;
        }
        self.set_hash();

        true
    }

    pub fn unmake_move(self: &mut Game, mv: Move) {
        let HistoryItem { unmove, hash } = self.history.pop().unwrap();

        let from = mv.from();
        let to = mv.to();
        let promoted = mv.promotion();
        let captured = ((unmove >> 32) & 0xff) as usize;
        self.state = (unmove & 0xffffffff) as usize;
        let side = self.state & 0xff;
        let ep = (self.state >> 16) & 0xff;
        let piece = if promoted != EMPTY { PAWN | side } else { self.board[to] };
        self.board[from] = piece;
        self.board[to] = captured;
        self.hash = hash;

        if piece == PAWN | WHITE {
            if to == ep {
                self.board[to + 10] = PAWN | BLACK;
            }
        } else if piece == PAWN | BLACK {
            if to == ep {
                self.board[to - 10] = PAWN | WHITE;
            }
        } else if piece == KING | WHITE {
            self.king_white = from;
            if from == 95 && to == 93 {
                self.board[91] = ROOK | WHITE;
                self.board[94] = EMPTY;
            } else if from == 95 && to == 97 {
                self.board[98] = ROOK | WHITE;
                self.board[96] = EMPTY;
            }
        } else if piece == KING | BLACK {
            self.king_black = from;
            if from == 25 && to == 23 {
                self.board[21] = ROOK | BLACK;
                self.board[24] = EMPTY;
            } else if from == 25 && to == 27 {
                self.board[28] = ROOK | BLACK;
                self.board[26] = EMPTY;
            }
        }
    }

    pub fn score_moves(self: &Game, move_list: &mut Vec<Move>, cutoff_moves: &[usize; 64 * 64], top_move: Move) {
        let tm = top_move & 0xffffffff;
        for mv in move_list.iter_mut() {
            let m = *mv & 0xffffffff;
            let score: usize = if m == tm {
                1000000000
            } else {
                let from = (m & 0xff) as usize;
                let to = ((m >> 8) & 0xff) as usize;
                let captured = self.board[to] & PIECE_MASK;
                if captured != EMPTY {
                    let piece = self.board[from] & PIECE_MASK;
                    1000000usize + captured * 10usize - piece
                } else {
                    cutoff_moves[REV8X8[from] * 64 + REV8X8[to]]
                }
            };
            *mv = (score as u64) << 32 | m;
        }
    }

    pub fn evaluate(self: &Game) -> isize {
        let mut white_pawn_mat: isize = 0;
        let mut white_piece_mat: isize = 0;
        let mut black_pawn_mat: isize = 0;
        let mut black_piece_mat: isize = 0;
        let mut white_pawn_rank: [usize; 10] = [0; 10];
        let mut black_pawn_rank: [usize; 10] = [7; 10];

        for i in 0..64 {
            let pos = MAP8X8[i];
            let piece = self.board[pos];
            match piece {
                WHITE_PAWN => {
                    white_pawn_mat += PAWN_VALUE;
                    let f = pos % 10;
                    white_pawn_rank[f] = std::cmp::max(white_pawn_rank[f], i / 8);
                },
                WHITE_BISHOP => {
                    white_piece_mat += BISHOP_VALUE;
                },
                WHITE_KNIGHT => {
                    white_piece_mat += KNIGHT_VALUE;
                },
                WHITE_ROOK => {
                    white_piece_mat += ROOK_VALUE;
                },
                WHITE_QUEEN => {
                    white_piece_mat += QUEEN_VALUE;
                },
                BLACK_PAWN => {
                    black_pawn_mat += PAWN_VALUE;
                    let f = pos % 10;
                    black_pawn_rank[f] = std::cmp::min(black_pawn_rank[f], i / 8);
                },
                BLACK_BISHOP => {
                    black_piece_mat += BISHOP_VALUE;
                },
                BLACK_KNIGHT => {
                    black_piece_mat += KNIGHT_VALUE;
                },
                BLACK_ROOK => {
                    black_piece_mat += ROOK_VALUE;
                },
                BLACK_QUEEN => {
                    black_piece_mat += QUEEN_VALUE;
                },
                _ => continue
            };
        }

        let mut white_score: isize = white_piece_mat + white_pawn_mat;
        let mut black_score: isize = black_piece_mat + black_pawn_mat;

        for i in 0..64 {
            let piece = self.board[MAP8X8[i]];
            match piece {
                WHITE_PAWN => {
                    white_score += evaluate_white_pawn(i, &white_pawn_rank, &black_pawn_rank);
                },
                WHITE_BISHOP => {
                    white_score += BISHOP_PCSQ[i];
                },
                WHITE_KNIGHT => {
                    white_score += KNIGHT_PCSQ[i];
                },
                WHITE_ROOK => {
                    if white_pawn_rank[i % 8 + 1] == 0 {
                        white_score += if black_pawn_rank[i % 8 + 1] == 7 {
                            ROOK_OPEN_FILE_BONUS
                        } else {
                            ROOK_SEMI_OPEN_FILE_BONUS
                        }
                    }
                    if i / 8 == 1 {
                        white_score += ROOK_ON_SEVENTH_BONUS;
                    }
                },
                BLACK_PAWN => {
                    black_score += evaluate_black_pawn(i, &white_pawn_rank, &black_pawn_rank);
                },
                BLACK_BISHOP => {
                    black_score += BISHOP_PCSQ[FLIP[i]];
                },
                BLACK_KNIGHT => {
                    black_score += KNIGHT_PCSQ[FLIP[i]];
                },
                BLACK_ROOK => {
                    if black_pawn_rank[i % 8 + 1] == 7 {
                        black_score += if white_pawn_rank[i % 8 + 1] == 0 {
                            ROOK_OPEN_FILE_BONUS
                        } else {
                            ROOK_SEMI_OPEN_FILE_BONUS
                        }
                    }
                    if i / 8 == 6 {
                        black_score += ROOK_ON_SEVENTH_BONUS;
                    }
                },
                _ => continue
            };
        }

        let side = self.state & 0xff;
        if side == WHITE {
            white_score - black_score
        } else {
            black_score - white_score
        }
    }

    pub fn in_check(self: &Game) -> bool {
        let side = self.state & 0xff;
        let king_position = if side == WHITE { self.king_white } else { self.king_black };
        let xside = if side == WHITE { BLACK } else { WHITE };
        self.is_attacked_by(king_position, xside)
    }

    pub fn white_to_move(self: &Game) -> bool {
        self.state & 0xff == WHITE
    }

    pub fn fifty_move_draw(self: &Game) -> bool {
        self.state.draw_ply() >= 100
    }

    pub fn repetitions(self: &Game) -> usize {
        let mut reps = 0;
        let length = self.history.len();
        let fifty = self.state.draw_ply();
        if fifty > length {
            return 0;
        }
        for k in (length - fifty)..length {
            if self.history[k].hash == self.hash {
                reps += 1;
            }
        }
        reps
    }

}

pub fn millis_since(time: &Instant) -> u64 {
    let elapsed = time.elapsed();
    return 1000 * elapsed.as_secs() + elapsed.subsec_millis() as u64;
}

pub trait ThinkInfo {
    fn think_info(&mut self, depth: usize, score: isize, mate_in: isize, node_count: usize,
                  millis: u64, moves: &Vec<String>);
}

pub struct Search<'a, T: ThinkInfo> {
    game: &'a mut Game,
    comms: &'a mut T,
    nodes: usize,
    start_time: Instant,
    max_millis: u64,
    pv: Vec<Vec<Move>>,
    tmp_pv: Vec<Move>,
    stop_thinking: bool,
    cutoff_moves: [usize; 64 * 64],
}

impl<'a, T: ThinkInfo> Search<'a, T> {

    pub fn new(game: &'a mut Game, max_millis: u64, comms: &'a mut T) -> Search<'a, T> {
        let mut pv: Vec<Vec<Move>> = Vec::with_capacity(MAX_DEPTH + 1);
        for _ in 0..(MAX_DEPTH + 1) {
            pv.push(Vec::with_capacity(MAX_DEPTH + 1));
        }
        Search {
            game,
            comms,
            nodes: 0,
            start_time: Instant::now(),
            max_millis,
            pv,
            tmp_pv: Vec::with_capacity(MAX_DEPTH + 1),
            stop_thinking: false,
            cutoff_moves: [0; 64 * 64],
        }
    }

    pub fn quiesce(self: &mut Search<'a, T>, alpha: isize, beta: isize,
                   ply: usize, follow_pv: bool) -> isize {
        self.nodes += 1;

        if self.nodes % 1024 == 0 && millis_since(&self.start_time) >= self.max_millis {
            self.stop_thinking = true;
            return 0;  // return value will be ignored
        }

        let mut score = self.game.evaluate();
        let mut alpha = alpha;

        if ply == MAX_DEPTH - 1 {
            return score;
        }
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }

        let mut moves = self.game.capture_moves();
        let mut follow_pv = follow_pv && ply < self.pv[0].len();
        self.game.score_moves(&mut moves, &self.cutoff_moves, if follow_pv { self.pv[0][ply] } else { DUMMY_MOVE });

        moves.sort_unstable_by(|a, b| (*b >> 32).cmp(&(*a >> 32)));

        for mv in moves {
            if !self.game.make_move(mv) {
                continue;
            }

            self.pv[ply + 1].clear();    
            score = -self.quiesce(-beta, -alpha, ply + 1, follow_pv);

            self.game.unmake_move(mv);

            if self.stop_thinking {
                return 0;  // return value will be ignored
            }
            if score > alpha {
                if score >= beta {
                    return beta;
                }
                alpha = score;
                
                self.tmp_pv.push(mv);
                self.tmp_pv.append(&mut self.pv[ply + 1]);
                self.pv[ply].clear();
                self.pv[ply].append(&mut self.tmp_pv);
            }
            follow_pv = false;
        }

        alpha
    }

    pub fn search(self: &mut Search<'a, T>, alpha: isize, beta: isize,
                  ply: usize, depth: usize, follow_pv: bool) -> isize {
        if cfg!(not(noquiesce)) {
            if ply >= depth {
                return self.quiesce(alpha, beta, ply, follow_pv);
            }
        }

        self.nodes += 1;

        if self.nodes % 1024 == 0 && millis_since(&self.start_time) >= self.max_millis {
            self.stop_thinking = true;
            return 0;  // return value will be ignored
        }

        if ply > 0 && self.game.repetitions() > 0 {
            // three-fold draw is not until the same position has been seen two
            // times before, but repeating a position is looking for a draw...
            return 0;
        }

        if cfg!(noquiesce) {
            if ply >= depth {
                return self.game.evaluate();
            }
        }
        if ply == MAX_DEPTH - 1 {
            return self.game.evaluate();
        }

        let mut moves = self.game.generate_moves();
        let mut any_legal_moves = false;
        let mut alpha = alpha;
        let mut depth = depth;
        let in_check = self.game.in_check();

        if in_check {
            depth += 1;
        }

        let mut follow_pv = follow_pv && ply < self.pv[0].len();
        self.game.score_moves(&mut moves, &self.cutoff_moves, if follow_pv { self.pv[0][ply] } else { DUMMY_MOVE });

        moves.sort_unstable_by(|a, b| (*b >> 32).cmp(&(*a >> 32)));

        for mv in moves {

            if !self.game.make_move(mv) {
                continue;
            }
            any_legal_moves = true;

            self.pv[ply + 1].clear();    
            let score = -self.search(-beta, -alpha, ply + 1, depth, follow_pv);

            self.game.unmake_move(mv);

            if self.stop_thinking {
                return 0;  // return value will be ignored
            }
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;

                self.cutoff_moves[REV8X8[mv.from()] * 64 + REV8X8[mv.to()]] += MAX_DEPTH - ply;

                self.tmp_pv.push(mv);
                self.tmp_pv.append(&mut self.pv[ply + 1]);
                self.pv[ply].clear();
                self.pv[ply].append(&mut self.tmp_pv);

                if ply == 0 {
                    let mate_in: isize = if score <= -(MATE_VALUE - MAX_DEPTH as isize) {
                        -(MATE_VALUE + score) / 2
                    } else if score >= (MATE_VALUE - MAX_DEPTH as isize) {
                        (MATE_VALUE - score) / 2
                    } else {
                        0
                    };
                    let millis = millis_since(&self.start_time);
                    let moves = self.pv[0].iter().map(|m| m.to_algebraic()).collect::<Vec<String>>();
                    self.comms.think_info(depth, score, mate_in, self.nodes, millis, &moves);
                }
            }
            follow_pv = false;
        }

        if !any_legal_moves {
            return if in_check { -MATE_VALUE + ply as isize } else { 0 };
        }

        if self.game.fifty_move_draw() {
            return 0;
        }

        alpha
    }
}

pub fn think<T: ThinkInfo>(game: &mut Game, millis_to_think: u64, search_depth: usize, comms: &mut T) -> Option<Move> {
    let mut search = Search::new(game, millis_to_think, comms);

    for depth in 1..(search_depth + 1) {
        let score = search.search(-MATE_VALUE, MATE_VALUE, 0, depth, true);
        if search.stop_thinking {
            break;
        }
        if score >= MATE_VALUE - MAX_DEPTH as isize {
            break;
        }
    }

    if search.pv[0].len() > 0 {
        Some(search.pv[0][0])
    } else {
        None
    }
}

fn legal_moves(game: &mut Game) -> Vec<Move> {
    let mut result: Vec<Move> = Vec::new();
    let move_list = game.generate_moves();
    for mv in move_list {
        if game.make_move(mv) {
            game.unmake_move(mv);
            result.push(mv);
        }
    }
    result
}

pub fn make_move_algebraic(game: &mut Game, input_move: &str) {
    let input_move = algebraic_to_move(input_move);
    let moves = legal_moves(game);
    for mv in moves {
        if mv == input_move {
            game.make_move(mv);
            return;
        }
    }
    panic!("make_move_algebraic");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_fold_repetition() {
        let mut game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0").unwrap();
        make_move_algebraic(&mut game, "g1f3");
        make_move_algebraic(&mut game, "b8c6");
        make_move_algebraic(&mut game, "f3g1");
        make_move_algebraic(&mut game, "c6b8");
        assert_eq!(game.repetitions(), 1);
        make_move_algebraic(&mut game, "g1f3");
        make_move_algebraic(&mut game, "b8c6");
        make_move_algebraic(&mut game, "f3g1");
        make_move_algebraic(&mut game, "c6b8");
        assert_eq!(game.repetitions(), 2);
        make_move_algebraic(&mut game, "g1f3");
        make_move_algebraic(&mut game, "b8c6");
        make_move_algebraic(&mut game, "f3g1");
        make_move_algebraic(&mut game, "c6b8");
        assert_eq!(game.repetitions(), 3);
    }
}

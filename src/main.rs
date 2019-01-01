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

// const MAX_SEARCH_DEPTH: usize = 32;
// const MAX_MOVES_PER_POSITION: usize = 218;

// #[derive(Copy, Clone)]
struct Move {
    m: usize,  // promoted << 16 | to << 8 | from
}

type UnMove = u64;  // captured << 32 | state

struct Game {
    board: [usize; 120],
    state: usize,  // draw_ply << 24 | ep << 16 | castling << 8 | turn
    king_white: usize,
    king_black: usize,
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
        state: 0,
        king_white: 0,
        king_black: 0,
    };
    for i in 0..64 {
        game.board[MAILBOX[i]] = EMPTY;
    }
    game
}

fn algebraic_to_pos(s: &str) -> Option<usize> {
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

const PIECE_ASCII: &str = " PBNRQKpbnrqk";
const PIECE_VALUES: [usize; 13] = [
    EMPTY,
    WHITE | PAWN, WHITE | BISHOP, WHITE | KNIGHT, WHITE | ROOK, WHITE | QUEEN, WHITE | KING,
    BLACK | PAWN, BLACK | BISHOP, BLACK | KNIGHT, BLACK | ROOK, BLACK | QUEEN, BLACK | KING
];

fn init_game(fen: &str) -> Result<Game, &str> {
    let mut iter = fen.split_whitespace();
    let mut game = new_game();
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
                        game.board[MAILBOX[pos]] = PIECE_VALUES[idx];
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
                Some(p) => MAILBOX[p],
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
    return Result::Ok(game);
}

const BISHOP_MOVEMENTS: [isize; 4] = [-11, -9, 9, 11];
const KNIGHT_MOVEMENTS: [isize; 8] = [-21, -19, -12, -8, 8, 12, 19, 21];
const ROOK_MOVEMENTS: [isize; 4] = [-1, 1, -10, 10];
const KING_MOVEMENTS: [isize; 8] = [-1, 1, -10, 10, -11, -9, 9, 11];

fn is_attacked_by(game: &Game, pos: usize, color: usize) -> bool {
    if color == WHITE {
        if game.board[pos + 9] == (PAWN | WHITE) || game.board[pos + 11] == (PAWN | WHITE) {
            return true;
        }
    } else {
        if game.board[pos - 9] == (PAWN | BLACK) || game.board[pos - 11] == (PAWN | BLACK) {
            return true;
        }
    }
    for delta in BISHOP_MOVEMENTS.iter() {
        let mut to = ((pos as isize) + delta) as usize;
        while game.board[to] == EMPTY {
            to = ((to as isize) + delta) as usize;
        }
        if game.board[to] == BISHOP | color {
            return true;
        }
    }
    for delta in KNIGHT_MOVEMENTS.iter() {
        if game.board[((pos as isize) + delta) as usize] == KNIGHT | color {
            return true;
        }
    }
    for delta in ROOK_MOVEMENTS.iter() {
        let mut to = ((pos as isize) + delta) as usize;
        while game.board[to] == EMPTY {
            to = ((to as isize) + delta) as usize;
        }
        if game.board[to] == ROOK | color {
            return true;
        }
    }
    for delta in KING_MOVEMENTS.iter() {
        let mut to = ((pos as isize) + delta) as usize;
        while game.board[to] == EMPTY {
            to = ((to as isize) + delta) as usize;
        }
        if game.board[to] == QUEEN | color {
            return true;
        }
    }
    for delta in KING_MOVEMENTS.iter() {
        if game.board[((pos as isize) + delta) as usize] == KING | color {
            return true;
        }
    }
    return false;
}

fn add_move(move_list: &mut Vec<Move>, from: usize, to: usize) {
    move_list.push(Move {
        m: to << 8 | from
    });
}

fn add_promotion(move_list: &mut Vec<Move>, from: usize, to: usize, side: usize) {
    move_list.push(Move {
        m: (BISHOP | side) << 16 | to << 8 | from
    });
    move_list.push(Move {
        m: (KNIGHT | side) << 16 | to << 8 | from
    });
    move_list.push(Move {
        m: (ROOK | side) << 16 | to << 8 | from
    });
    move_list.push(Move {
        m: (QUEEN | side) << 16 | to << 8 | from
    });
}

fn generate_moves(game: &Game, move_list: &mut Vec<Move>) {
    let side = game.state & 0xff;
    let xside = if side == WHITE { BLACK } else { WHITE };
    let castling = (game.state >> 8) & 0xff;
    let ep = (game.state >> 16) & 0xff;

    for i in 0..64 {
        let from = MAILBOX[i];
        let piece = game.board[from];
        if piece & COLOR_MASK == side {
            let base_piece = piece & PIECE_MASK;
            if base_piece == PAWN {
                if side == WHITE {
                    if i >> 3 == 1 {
                        if game.board[from - 10] == EMPTY {
                            add_promotion(move_list, from, from - 10, WHITE);
                        }
                        if game.board[from - 11] & COLOR_MASK == BLACK {
                            add_promotion(move_list, from, from - 11, WHITE);
                        }
                        if game.board[from - 9] & COLOR_MASK == BLACK {
                            add_promotion(move_list, from, from - 9, WHITE);
                        }
                    } else {
                        if game.board[from - 10] == EMPTY {
                            add_move(move_list, from, from - 10);
                            if i >> 3 == 6 && game.board[from - 20] == EMPTY {
                                add_move(move_list, from, from - 20);
                            }
                        }
                        if game.board[from - 11] & COLOR_MASK == BLACK || from - 11 == ep {
                            add_move(move_list, from, from - 11);
                        }
                        if game.board[from - 9] & COLOR_MASK == BLACK || from - 9 == ep {
                            add_move(move_list, from, from - 9);
                        }
                    }
                } else {  // side == BLACK
                    if i >> 3 == 6 {
                        if game.board[from + 10] == EMPTY {
                            add_promotion(move_list, from, from + 10, BLACK);
                        }
                        if game.board[from + 11] & COLOR_MASK == WHITE {
                            add_promotion(move_list, from, from + 11, BLACK);
                        }
                        if game.board[from + 9] & COLOR_MASK == WHITE {
                            add_promotion(move_list, from, from + 9, BLACK);
                        }
                    } else {
                        if game.board[from + 10] == EMPTY {
                            add_move(move_list, from, from + 10);
                            if i >> 3 == 1 && game.board[from + 20] == EMPTY {
                                add_move(move_list, from, from + 20);
                            }
                        }
                        if game.board[from + 11] & COLOR_MASK == WHITE || from + 11 == ep {
                            add_move(move_list, from, from + 11);
                        }
                        if game.board[from + 9] & COLOR_MASK == WHITE || from + 9 == ep {
                            add_move(move_list, from, from + 9);
                        }
                    }
                }
            } else if base_piece == BISHOP {
                for delta in BISHOP_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while game.board[to] == EMPTY {
                        add_move(move_list, from, to);
                        to = ((to as isize) + delta) as usize;
                    }
                    if game.board[to] & COLOR_MASK == xside {
                        add_move(move_list, from, to);
                    }
                }
            } else if base_piece == KNIGHT {
                for delta in KNIGHT_MOVEMENTS.iter() {
                    let to = ((from as isize) + delta) as usize;
                    if game.board[to] == EMPTY || game.board[to] & COLOR_MASK == xside {
                        add_move(move_list, from, to);
                    }
                }
            } else if base_piece == ROOK {
                for delta in ROOK_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while game.board[to] == EMPTY {
                        add_move(move_list, from, to);
                        to = ((to as isize) + delta) as usize;
                    }
                    if game.board[to] & COLOR_MASK == xside {
                        add_move(move_list, from, to);
                    }
                }
            } else if base_piece == QUEEN {
                for delta in KING_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while game.board[to] == EMPTY {
                        add_move(move_list, from, to);
                        to = ((to as isize) + delta) as usize;
                    }
                    if game.board[to] & COLOR_MASK == xside {
                        add_move(move_list, from, to);
                    }
                }
            } else if base_piece == KING {
                if from == 95 && (castling & (CASTLING_QUEEN_WHITE | CASTLING_KING_WHITE)) != 0
                        && !is_attacked_by(game, 95, BLACK) {
                    if (castling & CASTLING_QUEEN_WHITE) != 0
                            && game.board[94] == EMPTY && game.board[93] == EMPTY && game.board[92] == EMPTY
                            && !is_attacked_by(game, 94, BLACK) {
                        add_move(move_list, 95, 93);
                    }
                    if (castling & CASTLING_KING_WHITE) != 0
                            && game.board[96] == EMPTY && game.board[97] == EMPTY
                            && !is_attacked_by(game, 96, BLACK) {
                        add_move(move_list, 95, 97);
                    }
                } else if from == 25 && (castling & (CASTLING_QUEEN_BLACK | CASTLING_KING_BLACK)) != 0
                        && !is_attacked_by(game, 25, WHITE) {
                    if (castling & CASTLING_QUEEN_BLACK) != 0
                            && game.board[24] == EMPTY && game.board[23] == EMPTY && game.board[22] == EMPTY
                            && !is_attacked_by(game, 24, WHITE) {
                        add_move(move_list, 25, 23);
                    }
                    if (castling & CASTLING_KING_BLACK) != 0
                            && game.board[26] == EMPTY && game.board[27] == EMPTY
                            && !is_attacked_by(game, 26, WHITE) {
                        add_move(move_list, 25, 27);
                    }
              }
                for delta in KING_MOVEMENTS.iter() {
                    let to = ((from as isize) + delta) as usize;
                    if game.board[to] == EMPTY || game.board[to] & COLOR_MASK == xside {
                        add_move(move_list, from, to);
                    }
                }
            }
        }
    }
}

fn make_move(game: &mut Game, mv: &Move) -> Option<UnMove> {
    let from = mv.m & 0xff;
    let to = (mv.m >> 8) & 0xff;
    let promoted = (mv.m >> 16) & 0xff;
    let piece = game.board[from];
    let captured = game.board[to];
    game.board[to] = if promoted != EMPTY { promoted } else { piece };
    game.board[from] = EMPTY;
    let from_state = game.state;
    let from_ep = (from_state >> 16) & 0xff;
    let mut to_ep = 0;
    let from_draw_ply = (from_state >> 24) & 0xff;
    let mut to_draw_ply = if captured == EMPTY { from_draw_ply + 1 } else { 0 };
    let from_castling = (from_state >> 8) & 0xff;
    let to_castling = from_castling & CASTLE_MASK[from] & CASTLE_MASK[to];
    let side = from_state & 0xff;
    let xside = if side == WHITE { BLACK } else { WHITE };

    if piece == PAWN | WHITE {
        if to == from_ep {
            game.board[to + 10] = EMPTY;
        } else if to == from - 20 {
            to_ep = from - 10;
        }
        to_draw_ply = 0;
    } else if piece == PAWN | BLACK {
        if to == from_ep {
            game.board[to - 10] = EMPTY;
        } else if to == from + 20 {
            to_ep = from + 10;
        }
        to_draw_ply = 0;
    } else if piece == KING | WHITE {
        game.king_white = to;
        if from == 95 {
            if to == 93 {
                game.board[91] = EMPTY;
                game.board[94] = ROOK | WHITE;
            } else if to == 97 {
                game.board[98] = EMPTY;
                game.board[96] = ROOK | WHITE;
            }
        }
    } else if piece == KING | BLACK {
        game.king_black = to;
        if from == 25 {
            if to == 23 {
                game.board[21] = EMPTY;
                game.board[24] = ROOK | BLACK;
            } else if to == 27 {
                game.board[28] = EMPTY;
                game.board[26] = ROOK | BLACK;
            }
        }
    }

    game.state = to_draw_ply << 24 | to_ep << 16 | to_castling << 8 | xside;

    let unmove = (captured as u64) << 32 | from_state as u64;

    if is_attacked_by(game, if side == WHITE { game.king_white } else { game.king_black }, xside) {
        unmake_move(game, mv, unmove);
        return Option::None;
    }

    return Option::Some(unmove);
}

fn unmake_move(game: &mut Game, mv: &Move, umv: UnMove) {
    let from = mv.m & 0xff;
    let to = (mv.m >> 8) & 0xff;
    let promoted = (mv.m >> 16) & 0xff;
    let captured = ((umv >> 32) & 0xff) as usize;
    game.state = (umv & 0xffffffff) as usize;
    let side = game.state & 0xff;
    let ep = (game.state >> 16) & 0xff;
    let piece = if promoted != EMPTY { PAWN | side } else { game.board[to] };
    game.board[from] = piece;
    game.board[to] = captured;

    if piece == PAWN | WHITE {
        if to == ep {
            game.board[to + 10] = PAWN | BLACK;
        }
    } else if piece == PAWN | BLACK {
        if to == ep {
            game.board[to - 10] = PAWN | WHITE;
        }
    } else if piece == KING | WHITE {
        game.king_white = from;
        if from == 95 && to == 93 {
            game.board[91] = ROOK | WHITE;
            game.board[94] = EMPTY;
        } else if from == 95 && to == 97 {
            game.board[98] = ROOK | WHITE;
            game.board[96] = EMPTY;
        }
    } else if piece == KING | BLACK {
        game.king_black = from;
        if from == 25 && to == 23 {
            game.board[21] = ROOK | BLACK;
            game.board[24] = EMPTY;
        } else if from == 25 && to == 27 {
            game.board[28] = ROOK | BLACK;
            game.board[26] = EMPTY;
        }
    }
}

fn perft_sub(game: &mut Game, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut move_list = Vec::new();
    generate_moves(game, &mut move_list);

    let mut result = 0;
    for mv in &move_list {
        match make_move(game, &mv) {
            Some(umv) => {
                let count_sub = perft_sub(game, depth - 1);
                unmake_move(game, &mv, umv);
                result += count_sub;
            },
            None => {}
        }
    }

    result
}

fn perft(fen: &str, depth: usize) -> usize {
    match init_game(fen) {
        Ok(mut game) => perft_sub(&mut game, depth),
        Err(_) => 0
    }
}

fn main() {
    println!("Perft: {}", perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 4));
}

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
const CASTLING_KING_WHITE: u8 = 1;
const CASTLING_QUEEN_WHITE: u8 = 2;
const CASTLING_KING_BLACK: u8 = 3;
const CASTLING_QUEEN_BLACK: u8 = 4;

const MAX_SEARCH_DEPTH: usize = 32;
const MAX_MOVES_PER_POSITION: usize = 218;

#[derive(Copy, Clone)]
struct Move {
    from: u8,
    to: u8,
    // promoted: u8,
    // captured: u8,
    // from_castling: u8,
    // from_ep: u8,
    // from_draw_ply: u8
}

struct Game {
    board: [u8; 120],
    turn: u8,
    castling: u8,
    ep: u8,
    draw_ply: u8,
    king_white: u8,
    king_black: u8,
}

struct MoveContainer {
    move_list: [Move; MAX_SEARCH_DEPTH * MAX_MOVES_PER_POSITION],
    move_index: Vec<usize>,
    search_ply: usize,
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

const DUMMY_MOVE: Move = Move {
    from: 0,
    to: 0,
    // promoted: EMPTY,
    // captured: EMPTY,
    // from_castling: 0,
    // from_ep: 0,
    // from_draw_ply: 0
};

fn new_game() -> Game {
    let mut game = Game {
        board: [OFF_BOARD; 120],
        turn: 0,
        castling: 0,
        ep: 0,
        draw_ply: 0,
        king_white: 0,
        king_black: 0,
    };
    for i in 0..64 {
        game.board[MAILBOX[i] as usize] = EMPTY;
    }
    game
}

fn new_move_container() -> MoveContainer {
    let mut move_container = MoveContainer {
        move_list: [DUMMY_MOVE; MAX_SEARCH_DEPTH * MAX_MOVES_PER_POSITION],
        move_index: Vec::new(),
        search_ply: 0
    };
    move_container.move_index.push(0);
    move_container
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
    game.board[31] = PAWN | BLACK;
    game.board[32] = PAWN | BLACK;
    game.board[33] = PAWN | BLACK;
    game.board[34] = PAWN | BLACK;
    game.board[35] = PAWN | BLACK;
    game.board[36] = PAWN | BLACK;
    game.board[37] = PAWN | BLACK;
    game.board[38] = PAWN | BLACK;
    game.board[81] = PAWN | WHITE;
    game.board[82] = PAWN | WHITE;
    game.board[83] = PAWN | WHITE;
    game.board[84] = PAWN | WHITE;
    game.board[85] = PAWN | WHITE;
    game.board[86] = PAWN | WHITE;
    game.board[87] = PAWN | WHITE;
    game.board[88] = PAWN | WHITE;
    game.board[91] = ROOK | WHITE;
    game.board[92] = KNIGHT | WHITE;
    game.board[93] = BISHOP | WHITE;
    game.board[94] = QUEEN | WHITE;
    game.board[95] = KING | WHITE;
    game.board[96] = BISHOP | WHITE;
    game.board[97] = KNIGHT | WHITE;
    game.board[98] = ROOK | WHITE;
    game.turn = WHITE;
    game.castling = CASTLING_QUEEN_WHITE | CASTLING_KING_WHITE | CASTLING_QUEEN_BLACK | CASTLING_KING_BLACK;
    game.ep = 0;
    game.draw_ply = 0;
    game.king_black = 25;
    game.king_white = 95;
}

const BISHOP_MOVEMENTS: [isize; 4] = [-11, -9, 9, 11];
const KNIGHT_MOVEMENTS: [isize; 8] = [-21, -19, -12, -8, 8, 12, 19, 21];
const ROOK_MOVEMENTS: [isize; 4] = [-1, 1, -10, 10];
const KING_MOVEMENTS: [isize; 8] = [-1, 1, -10, 10, -11, -9, 9, 11];

fn generate_moves(game: &Game, move_container: &mut MoveContainer) {
    let side = game.turn;
    let xside = if side == WHITE { BLACK } else { WHITE };
    let board = game.board;
    let mut move_offset = move_container.move_index[move_container.search_ply];

    for i in 0..64 {
        let from = MAILBOX[i];
        let piece = board[from];
        if piece & COLOR_MASK == side {
            let base_piece = piece & PIECE_MASK;
            if base_piece == PAWN {
                if side == WHITE {
                    if board[from - 10] == EMPTY {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from - 10) as u8;
                        move_offset += 1;
                        if i >> 3 == 6 && board[from - 20] == EMPTY {
                            move_container.move_list[move_offset].from = from as u8;
                            move_container.move_list[move_offset].to = (from - 20) as u8;
                            move_offset += 1;
                        }
                    }
                    if board[from - 11] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from - 11) as u8;
                        move_offset += 1;
                    }
                    if board[from - 9] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from - 9) as u8;
                        move_offset += 1;
                    }
                } else {
                    if board[from + 10] == EMPTY {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from + 10) as u8;
                        move_offset += 1;
                        if i >> 3 == 1 && board[from + 20] == EMPTY {
                            move_container.move_list[move_offset].from = from as u8;
                            move_container.move_list[move_offset].to = (from + 10) as u8;
                            move_offset += 1;
                        }
                    }
                    if board[from + 11] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from + 11) as u8;
                        move_offset += 1;
                    }
                    if board[from + 9] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = (from + 9) as u8;
                        move_offset += 1;
                    }
                }
            } else if base_piece == BISHOP {
                for delta in BISHOP_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while board[to] == EMPTY {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                        to = ((to as isize) + delta) as usize;
                    }
                    if board[to] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                    }
                }
            } else if base_piece == KNIGHT {
                for delta in KNIGHT_MOVEMENTS.iter() {
                    let to = ((from as isize) + delta) as usize;
                    if board[to] == EMPTY || board[to] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                    }
                }
            } else if base_piece == ROOK {
                for delta in ROOK_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while board[to] == EMPTY {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                        to = ((to as isize) + delta) as usize;
                    }
                    if board[to] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                    }
                }
            } else if base_piece == QUEEN {
                for delta in KING_MOVEMENTS.iter() {
                    let mut to = ((from as isize) + delta) as usize;
                    while board[to] == EMPTY {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                        to = ((to as isize) + delta) as usize;
                    }
                    if board[to] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                    }
                }
            } else if base_piece == KING {
                for delta in KING_MOVEMENTS.iter() {
                    let to = ((from as isize) + delta) as usize;
                    if board[to] == EMPTY || board[to] & COLOR_MASK == xside {
                        move_container.move_list[move_offset].from = from as u8;
                        move_container.move_list[move_offset].to = to as u8;
                        move_offset += 1;
                    }
                }
            }
        }
    }
    move_container.move_index.push(move_offset);
}

fn main() {
    let mut game = new_game();
    let mut move_container = new_move_container();
    
    init_game(&mut game);

    generate_moves(&game, &mut move_container);

    for i in move_container.move_index[0]..move_container.move_index[1] {
        println!("Move {} {}", move_container.move_list[i].from, move_container.move_list[i].to);
    }
}

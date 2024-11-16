// This module contains logic to encode a piece move into a an encoded move.

use crate::piece::Piece;
use crate::piece_move::PieceMove;

// Represents the code for a given piece move, i.e, Piece<PieceMove>.
struct EcMove {
    row: u8,
    col: u8,
    code: u8,
}

impl From<Piece<PieceMove>> for EcMove {
    fn from(piece_move: Piece<PieceMove>) -> EcMove {
        let pm = piece_move.val();
        let (from_sq, to_sq) = pm.from_to();
        let (from_row, from_col) = from_sq.rc();
        let (to_row, to_col) = to_sq.rc();
        let row_diff = to_row as i8 - from_row as i8;
        let col_diff = to_col as i8 - from_col as i8;

        assert!(row_diff != 0 || col_diff != 0);

        let code = match piece_move.kind() {
            Piece::Knight(_) => 56 + encode_knight_move(row_diff, col_diff),
            Piece::Pawn(_) if pm.is_promo() && !pm.promo().unwrap().is_queen() => {
                64 + encode_under_promo(col_diff, pm.promo().unwrap())
            }
            _ => encode_queen_move(row_diff, col_diff),
        };

        Self {
            row: from_row,
            col: from_col,
            code,
        }
    }
}

// Converts a knight move to a number in the range [0, 7].
fn encode_knight_move(row_diff: i8, col_diff: i8) -> u8 {
    match (row_diff, col_diff) {
        // Lower left quadrant.
        (-2, -1) => 0,
        (-1, -2) => 1,
        // Upper left quadrant.
        (1, -2) => 2,
        (2, -1) => 3,
        // Upper right quadrant.
        (2, 1) => 4,
        (1, 2) => 5,
        // Lower right quadrant.
        (-1, 2) => 6,
        (-2, 1) => 7,
        _ => panic!(
            "row_diff={} and col_diff={} don't make sense for a knight move.",
            row_diff, col_diff
        ),
    }
}

// Converts an underpromotion move, i.e. a promotion to a piece other than a
// queen, to a number in the range [0, 8].
fn encode_under_promo(col_diff: i8, promo: Piece<()>) -> u8 {
    assert!(col_diff >= -1 && col_diff <= 1);

    let code = match promo {
        Piece::Rook(_) => 0,
        Piece::Bishop(_) => 3,
        Piece::Knight(_) => 6,
        _ => panic!("piece={:?} is not valid for under promotion.", promo),
    };

    match col_diff {
        0 => code + 1,
        1 => code + 2,
        _ => code,
    }
}

// Converts a queen move to a number in the range [0, 55].
fn encode_queen_move(row_diff: i8, col_diff: i8) -> u8 {
    assert!(row_diff >= -7 && row_diff <= 7);
    assert!(col_diff >= -7 && col_diff <= 7);
    assert!(col_diff != 0 || col_diff != 0);

    if row_diff < 0 {
        let nrows = (row_diff * -1) as u8;
        if col_diff == 0 {
            nrows
        } else if col_diff < 0 {
            nrows + 7
        } else {
            nrows + 14
        }
    } else if row_diff == 0 {
        if col_diff < 0 {
            21 + (col_diff * -1) as u8
        } else {
            28 + col_diff as u8
        }
    } else if col_diff == 0 {
        35 + row_diff as u8
    } else if col_diff < 0 {
        42 + row_diff as u8
    } else {
        49 + row_diff as u8
    }
}

// This module contains logic to encode a piece move into a an encoded move.

use crate::piece::Piece;
use crate::piece_move::PieceMove;
use std::cmp::max;

// Represents the code for a given piece move, i.e, Piece<PieceMove>.
pub(crate) struct EcMove {
    pub(crate) row: u8,
    pub(crate) col: u8,
    pub(crate) code: u8,
}

impl EcMove {
    // Converts the row and column back to a square index.
    pub fn sq_index(&self) -> u8 {
        self.row * 8 + self.col
    }

    // Maps EcMove to a number in [0, 8x8x73).
    pub fn index(&self) -> usize {
        (self.row as usize + 1) * (self.col as usize + 1) * self.code as usize - 1
    }
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

// Converts a knight move to a number in the range [1, 8].
fn encode_knight_move(row_diff: i8, col_diff: i8) -> u8 {
    match (row_diff, col_diff) {
        // Lower left quadrant.
        (-2, -1) => 1,
        (-1, -2) => 2,
        // Upper left quadrant.
        (1, -2) => 3,
        (2, -1) => 4,
        // Upper right quadrant.
        (2, 1) => 5,
        (1, 2) => 6,
        // Lower right quadrant.
        (-1, 2) => 7,
        (-2, 1) => 8,
        _ => panic!(
            "row_diff={} and col_diff={} don't make sense for a knight move.",
            row_diff, col_diff
        ),
    }
}

// Converts an underpromotion move, i.e. a promotion to a piece other than a
// queen, to a number in the range [1, 9].
fn encode_under_promo(col_diff: i8, promo: Piece<()>) -> u8 {
    assert!(col_diff >= -1 && col_diff <= 1);

    let code = match promo {
        Piece::Rook(_) => 1,
        Piece::Bishop(_) => 4,
        Piece::Knight(_) => 7,
        _ => panic!("piece={:?} is not valid for under promotion.", promo),
    };

    match col_diff {
        0 => code + 1,
        1 => code + 2,
        _ => code,
    }
}

// Converts a queen move to a number in the range [1, 56].
fn encode_queen_move(row_diff: i8, col_diff: i8) -> u8 {
    let direction = Direction::from_row_col_diff(row_diff, col_diff).to_u8();
    let num_sq = max(row_diff.unsigned_abs(), col_diff.unsigned_abs());
    direction * num_sq
}

// An enum to encode the direction in which the queen moves.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    // Creates the direction from a row_diff and col_diff, where the diffs represent
    // the difference between the destination and source squares.
    fn from_row_col_diff(row_diff: i8, col_diff: i8) -> Self {
        assert!(row_diff >= -7 && row_diff <= 7);
        assert!(col_diff >= -7 && col_diff <= 7);
        assert!(row_diff != 0 || col_diff != 0);

        if row_diff == 0 {
            if col_diff > 0 {
                Direction::East
            } else {
                Direction::West
            }
        } else if row_diff > 0 {
            if col_diff == 0 {
                Direction::North
            } else if col_diff > 0 {
                Direction::NorthEast
            } else {
                Direction::NorthWest
            }
        } else {
            if col_diff == 0 {
                Direction::South
            } else if col_diff > 0 {
                Direction::SouthEast
            } else {
                Direction::SouthWest
            }
        }
    }

    fn to_u8(&self) -> u8 {
        match *self {
            Direction::North => 1,
            Direction::NorthEast => 2,
            Direction::East => 3,
            Direction::SouthEast => 4,
            Direction::South => 5,
            Direction::West => 6,
            Direction::SouthWest => 7,
            Direction::NorthWest => 8,
        }
    }
}

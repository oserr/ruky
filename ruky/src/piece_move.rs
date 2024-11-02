use crate::bitboard::{BitErr, BitErr::*};
use crate::piece::Piece;
use crate::sq::Sq;
use serde::Serialize;
use std::convert::From;

// An enum for representing the different types of moves for all chess pieces.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum PieceMove {
    // Simple represents simple moves that only need a source and destination square to be fully
    // specified.
    Simple {
        from: Sq,
        to: Sq,
    },

    // For simple captures where the captured piece is on the destination square.
    Capture {
        from: Sq,
        to: Sq,
        cap: Piece<()>,
    },

    // Castling between king and rook, the only type of move where two pieces move simultaneously.
    Castle {
        king_from: Sq,
        king_to: Sq,
        rook_from: Sq,
        rook_to: Sq,
    },

    // Capture by en-passant.
    EnPassant {
        from: Sq,
        to: Sq,
        // The en-passant square, where the enemy pawn is located.
        passant: Sq,
    },

    // Pawn promotion.
    Promo {
        from: Sq,
        to: Sq,
        // The piece to which the pawn is promoted.
        promo: Piece<()>,
    },

    // Pawn promotion with capture.
    PromoCap {
        from: Sq,
        to: Sq,
        promo: Piece<()>,
        cap: Piece<()>,
    },
}

impl PieceMove {
    // Returns true if the PieceMove represents a capture.
    pub fn is_capture(&self) -> bool {
        matches!(
            *self,
            PieceMove::Capture { .. } | PieceMove::EnPassant { .. } | PieceMove::PromoCap { .. }
        )
    }

    // Returns true if the move represents a capture for a king.
    pub fn is_king_capture(&self) -> bool {
        match *self {
            PieceMove::Capture { cap, .. } | PieceMove::PromoCap { cap, .. } => cap.is_king(),
            _ => false,
        }
    }

    // Returns source and destination squares.
    pub fn from_to(&self) -> (Sq, Sq) {
        match *self {
            PieceMove::Simple { from, to } => (from, to),
            PieceMove::Capture { from, to, .. } => (from, to),
            PieceMove::Castle {
                king_from, king_to, ..
            } => (king_from, king_to),
            PieceMove::EnPassant { from, to, .. } => (from, to),
            PieceMove::Promo { from, to, .. } => (from, to),
            PieceMove::PromoCap { from, to, .. } => (from, to),
        }
    }

    // Returns true if the move type represents a pawn promotion.
    pub fn is_promo(&self) -> bool {
        matches!(*self, PieceMove::Promo { .. } | PieceMove::PromoCap { .. })
    }

    // Returns true if the move type represents a pawn promotion.
    pub fn promo(&self) -> Option<Piece<()>> {
        match *self {
            PieceMove::Promo { promo, .. } => Some(promo),
            PieceMove::PromoCap { promo, .. } => Some(promo),
            _ => None,
        }
    }
}

// Represents a move error.
#[derive(thiserror::Error, Clone, Debug)]
pub enum MoveErr {
    #[error("cannot promote to {0:?}")]
    BadPromo(Piece<()>),
    #[error("cannot move from square {0:?}")]
    BadFromSquare(Sq),
    #[error("cannot move to square {0:?}")]
    BadToSquare(Sq),
    #[error("bad move {0:?}")]
    BadMove(Piece<PieceMove>),
    #[error("move {0:?} does not represent a capture")]
    NoCapture(PieceMove),
}

// Conversion from BitErr to MoveErr.
impl From<BitErr> for MoveErr {
    fn from(err: BitErr) -> MoveErr {
        match err {
            IsNotSet(from) | FromIsNotSet(from) => MoveErr::BadFromSquare(from),
            IsSetAlready(to) | ToIsSetAlready(to) => MoveErr::BadToSquare(to),
        }
    }
}

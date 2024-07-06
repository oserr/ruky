// This module contains the utilities and types for representing UCI moves.

use crate::piece::Piece;
use crate::sq::Sq;

// Pm for [P]iece [m]ove.
pub enum Pm {
    // Represents a null move.
    Null,

    // A normal UCI move, with a source and destination square.
    Normal { from: Sq, to: Sq },

    // A pawn promotion move.
    Promo { from: Sq, to: Sq, promo: Piece },
}

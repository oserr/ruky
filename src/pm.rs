// This module contains the utilities and types for representing UCI moves.

use crate::err::UziErr;
use crate::piece::Piece;
use crate::sq::Sq;
use std::fmt::{self, Display, Formatter};

// Pm for [P]iece [m]ove.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pm {
    // Represents a null move.
    Null,

    // A normal UCI move, with a source and destination square.
    Normal { from: Sq, to: Sq },

    // A pawn promotion move.
    Promo { from: Sq, to: Sq, promo: Piece },
}

impl Display for Pm {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Pm::Null => formatter.write_str("0000"),
            Pm::Normal { from, to } => write!(formatter, "{}{}", from, to),
            Pm::Promo { from, to, promo } => write!(formatter, "{}{}{}", from, to, promo),
        }
    }
}

impl TryFrom<&[u8]> for Pm {
    type Error = UziErr;

    fn try_from(_bytes: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

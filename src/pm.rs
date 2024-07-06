// This module contains the utilities and types for representing UCI moves.

use crate::err::UziErr;
use crate::piece::Piece;
use crate::sq::Sq;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

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

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        match bytes.len() {
            4 if bytes[0] != b'0' => Ok(Pm::Normal {
                from: Sq::try_from(&bytes[..2])?,
                to: Sq::try_from(&bytes[2..])?,
            }),
            5 => Ok(Pm::Promo {
                from: Sq::try_from(&bytes[..2])?,
                to: Sq::try_from(&bytes[2..4])?,
                promo: Piece::try_from(bytes[4])?,
            }),
            4 if bytes.iter().all(|b| *b == b'0') => Ok(Pm::Null),
            _ => Err(UziErr::ParseMoveErr),
        }
    }
}

impl FromStr for Pm {
    type Err = UziErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Pm::try_from(s.as_bytes())
    }
}

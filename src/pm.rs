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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pm_from_null_move() {
        let null_move = &[b'0', b'0', b'0', b'0'];
        assert_eq!(Pm::try_from(&null_move[..]), Ok(Pm::Null));
        assert_eq!(Pm::from_str("0000"), Ok(Pm::Null));
    }

    #[test]
    fn pm_from_normal_move() {
        let e2e4 = &[b'e', b'2', b'e', b'4'];
        let pm = Pm::Normal {
            from: Sq::from((1, 4)),
            to: Sq::from((3, 4)),
        };

        assert_eq!(Pm::try_from(&e2e4[..]), Ok(pm));
        assert_eq!(Pm::from_str("e2e4"), Ok(pm));
    }

    #[test]
    fn pm_from_promo_move() {
        let promo_move = &[b'a', b'7', b'a', b'8', b'q'];
        let pm = Pm::Promo {
            from: Sq::from((6, 0)),
            to: Sq::from((7, 0)),
            promo: Piece::Queen,
        };

        assert_eq!(Pm::try_from(&promo_move[..]), Ok(pm));
        assert_eq!(Pm::from_str("a7a8q"), Ok(pm));
    }
}

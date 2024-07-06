// This module contains the definition for Piece, a basic enum to represent
// chess pieces.

use crate::err::UziErr;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Piece {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl Piece {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Piece::King => "k",
            Piece::Queen => "a",
            Piece::Rook => "r",
            Piece::Bishop => "b",
            Piece::Knight => "n",
            Piece::Pawn => "p",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Piece::King => b'k',
            Piece::Queen => b'a',
            Piece::Rook => b'r',
            Piece::Bishop => b'b',
            Piece::Knight => b'n',
            Piece::Pawn => b'p',
        }
    }

    pub fn to_char(&self) -> char {
        match *self {
            Piece::King => 'k',
            Piece::Queen => 'a',
            Piece::Rook => 'r',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            Piece::Pawn => 'p',
        }
    }

    pub fn is_king(&self) -> bool {
        matches!(self, Piece::King)
    }

    pub fn is_queen(&self) -> bool {
        matches!(self, Piece::Queen)
    }

    pub fn is_rook(&self) -> bool {
        matches!(self, Piece::Rook)
    }

    pub fn is_bishop(&self) -> bool {
        matches!(self, Piece::Bishop)
    }

    pub fn is_knight(&self) -> bool {
        matches!(self, Piece::Bishop)
    }

    pub fn is_pawn(&self) -> bool {
        matches!(self, Piece::Pawn)
    }
}

impl FromStr for Piece {
    type Err = UziErr;
    fn from_str(p: &str) -> Result<Self, Self::Err> {
        match p {
            "k" => Ok(Piece::King),
            "q" => Ok(Piece::Queen),
            "r" => Ok(Piece::Rook),
            "n" => Ok(Piece::Knight),
            "p" => Ok(Piece::Pawn),
            _ => Err(UziErr::ParsePieceErr(p.to_string())),
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = UziErr;
    fn try_from(p: char) -> Result<Self, Self::Error> {
        match p {
            'k' => Ok(Piece::King),
            'q' => Ok(Piece::Queen),
            'r' => Ok(Piece::Rook),
            'n' => Ok(Piece::Knight),
            'p' => Ok(Piece::Pawn),
            _ => Err(UziErr::ParsePieceErr(p.to_string())),
        }
    }
}

impl TryFrom<u8> for Piece {
    type Error = UziErr;
    fn try_from(p: u8) -> Result<Self, Self::Error> {
        match p {
            b'k' => Ok(Piece::King),
            b'q' => Ok(Piece::Queen),
            b'r' => Ok(Piece::Rook),
            b'n' => Ok(Piece::Knight),
            b'p' => Ok(Piece::Pawn),
            _ => Err(UziErr::ParsePieceErr(p.to_string())),
        }
    }
}

impl Display for Piece {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

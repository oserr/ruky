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
            Piece::Queen => "q",
            Piece::Rook => "r",
            Piece::Bishop => "b",
            Piece::Knight => "n",
            Piece::Pawn => "p",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Piece::King => b'k',
            Piece::Queen => b'q',
            Piece::Rook => b'r',
            Piece::Bishop => b'b',
            Piece::Knight => b'n',
            Piece::Pawn => b'p',
        }
    }

    pub fn to_char(&self) -> char {
        match *self {
            Piece::King => 'k',
            Piece::Queen => 'q',
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
            "b" => Ok(Piece::Bishop),
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
            'b' => Ok(Piece::Bishop),
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
            b'b' => Ok(Piece::Bishop),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_try_from_str() {
        let str_to_piece: [(&str, Piece); 6] = [
            ("k", Piece::King),
            ("q", Piece::Queen),
            ("r", Piece::Rook),
            ("b", Piece::Bishop),
            ("n", Piece::Knight),
            ("p", Piece::Pawn),
        ];
        for (s, p) in &str_to_piece {
            assert_eq!(Piece::from_str(*s), Ok(*p));
            assert_eq!(p.as_str(), *s);
        }
    }

    #[test]
    fn piece_try_from_char() {
        let char_to_piece: [(char, Piece); 6] = [
            ('k', Piece::King),
            ('q', Piece::Queen),
            ('r', Piece::Rook),
            ('b', Piece::Bishop),
            ('n', Piece::Knight),
            ('p', Piece::Pawn),
        ];
        for (c, p) in &char_to_piece {
            assert_eq!(Piece::try_from(*c), Ok(*p));
            assert_eq!(p.to_char(), *c);
        }
    }

    #[test]
    fn piece_try_from_u8() {
        let byte_to_piece: [(u8, Piece); 6] = [
            (b'k', Piece::King),
            (b'q', Piece::Queen),
            (b'r', Piece::Rook),
            (b'b', Piece::Bishop),
            (b'n', Piece::Knight),
            (b'p', Piece::Pawn),
        ];
        for (b, p) in &byte_to_piece {
            assert_eq!(Piece::try_from(*b), Ok(*p));
            assert_eq!(p.to_u8(), *b);
        }
    }
}

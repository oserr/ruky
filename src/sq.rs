// This module contains the defintion for Sq, a simple class to represent a
// square on the board.

use crate::err::UziErr;
use std::cmp::min;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct Sq {
    index: u8,
}

impl Sq {
    #[inline]
    // Creates a new Sq from index. If index is not a valid square index in range
    // [0, 63], it uses index 63.
    pub fn new(index: u8) -> Self {
        Sq {
            index: min(index, MAX),
        }
    }

    // Returns a tuple (row, col) with the row and column.
    #[inline]
    pub fn rc(&self) -> (u8, u8) {
        (self.index / 8, self.index % 8)
    }

    // We can write a shorter function to compute a string by converting the self to
    // (row, col), but being able to return a &str means this function is faster
    // and we don't need to allocate anything on the heap to simply get the str
    // version of the square.
    pub fn as_str(&self) -> &'static str {
        match self.index {
            0 => "a1",
            1 => "b1",
            2 => "c1",
            3 => "d1",
            4 => "e1",
            5 => "f1",
            6 => "g1",
            7 => "h1",
            8 => "a2",
            9 => "b2",
            10 => "c2",
            11 => "d2",
            12 => "e2",
            13 => "f2",
            14 => "g2",
            15 => "h2",
            16 => "a3",
            17 => "b3",
            18 => "c3",
            19 => "d3",
            20 => "e3",
            21 => "f3",
            22 => "g3",
            23 => "h3",
            24 => "a4",
            25 => "b4",
            26 => "c4",
            27 => "d4",
            28 => "e4",
            29 => "f4",
            30 => "g4",
            31 => "h4",
            32 => "a5",
            33 => "b5",
            34 => "c5",
            35 => "d5",
            36 => "e5",
            37 => "f5",
            38 => "g5",
            39 => "h5",
            40 => "a6",
            41 => "b6",
            42 => "c6",
            43 => "d6",
            44 => "e6",
            45 => "f6",
            46 => "g6",
            47 => "h6",
            48 => "a7",
            49 => "b7",
            50 => "c7",
            51 => "d7",
            52 => "e7",
            53 => "f7",
            54 => "g7",
            55 => "h7",
            56 => "a8",
            57 => "b8",
            58 => "c8",
            59 => "d8",
            60 => "e8",
            61 => "f8",
            62 => "g8",
            63 => "h8",
            _ => panic!("{} is not a valid square number", self.index),
        }
    }
}

impl FromStr for Sq {
    type Err = UziErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Sq::try_from(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for Sq {
    type Error = UziErr;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 2 {
            return Err(UziErr::ParseSqErr);
        }

        let col = match bytes[0] {
            c @ b'a'..=b'h' => c - b'a',
            _ => return Err(UziErr::ParseSqErr),
        };

        let row = match bytes[1] {
            r @ b'1'..=b'8' => r - b'1',
            _ => return Err(UziErr::ParseSqErr),
        };

        Ok(Sq::from((row, col)))
    }
}

impl Display for Sq {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

macro_rules! from_sq_for_integral {
    ( $( $t:ident )* ) => {
        $(
            impl From<Sq> for $t {
                fn from(sq: Sq) -> $t {
                    sq.index as $t
                }
            }
        )*
    }
}

// Generate trait implementations for From<Sq> for all integral types up to i64
// and u64.
from_sq_for_integral![u8 u16 u32 u64 i8 i16 i32 i64];

impl From<(u8, u8)> for Sq {
    fn from((row, col): (u8, u8)) -> Sq {
        Sq::new(row * 8 + col)
    }
}

// A handy constant to represnt the last valid square.
const MAX: u8 = 63;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_sq() {
        assert_eq!(u8::from(Sq::new(8)), 8u8);
        assert_eq!(u8::from(Sq::new(100)), 63u8);
    }

    #[test]
    fn sq_as_str() {
        assert_eq!(Sq::new(4).as_str(), "e1");
        assert_eq!(Sq::from((1, 4)).as_str(), "e2");
        assert_eq!(Sq::new(63).as_str(), "h8");
    }

    #[test]
    fn sq_from_str() {
        assert_eq!(Sq::from_str("e1"), Ok(Sq::new(4)));
        assert_eq!(Sq::from_str("eq3"), Err(UziErr::ParseSqErr));
    }

    #[test]
    fn sq_from_bytes() {
        assert_eq!(Sq::try_from(&[b'e', b'1'][..]), Ok(Sq::new(4)));
        assert_eq!(
            Sq::try_from(&[b'e', b'q', b'3'][..]),
            Err(UziErr::ParseSqErr)
        );
    }
}

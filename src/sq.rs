// This module contains the defintion for Sq, a simple class to represent a
// square on the board.

use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq, PartialOrd)]
struct Sq {
    index: u8,
}

impl Sq {
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

impl Debug for Sq {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
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

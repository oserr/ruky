use std::convert::From;
use std::fmt::{self, Debug, Formatter};
use std::ops::Shl;

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Sq {
    val: u8,
}

impl Sq {
    // Returns the row and column for the square as a pair (row, column).
    pub fn rc(&self) -> (u8, u8) {
        (self.val / 8, self.val % 8)
    }

    pub fn str(&self) -> &'static str {
        match self.val {
            0 => &"A1",
            1 => &"B1",
            2 => &"C1",
            3 => &"D1",
            4 => &"E1",
            5 => &"F1",
            6 => &"G1",
            7 => &"H1",
            8 => &"A2",
            9 => &"B2",
            10 => &"C2",
            11 => &"D2",
            12 => &"E2",
            13 => &"F2",
            14 => &"G2",
            15 => &"H2",
            16 => &"A3",
            17 => &"B3",
            18 => &"C3",
            19 => &"D3",
            20 => &"E3",
            21 => &"F3",
            22 => &"G3",
            23 => &"H3",
            24 => &"A4",
            25 => &"B4",
            26 => &"C4",
            27 => &"D4",
            28 => &"E4",
            29 => &"F4",
            30 => &"G4",
            31 => &"H4",
            32 => &"A5",
            33 => &"B5",
            34 => &"C5",
            35 => &"D5",
            36 => &"E5",
            37 => &"F5",
            38 => &"G5",
            39 => &"H5",
            40 => &"A6",
            41 => &"B6",
            42 => &"C6",
            43 => &"D6",
            44 => &"E6",
            45 => &"F6",
            46 => &"G6",
            47 => &"H6",
            48 => &"A7",
            49 => &"B7",
            50 => &"C7",
            51 => &"D7",
            52 => &"E7",
            53 => &"F7",
            54 => &"G7",
            55 => &"H7",
            56 => &"A8",
            57 => &"B8",
            58 => &"C8",
            59 => &"D8",
            60 => &"E8",
            61 => &"F8",
            62 => &"G8",
            63 => &"H8",
            _ => panic!("{} is not a valid square number", self.val),
        }
    }
}

impl Debug for Sq {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.str())
    }
}

impl From<Sq> for &'static str {
    fn from(sq: Sq) -> &'static str {
        sq.str()
    }
}

impl From<usize> for Sq {
    fn from(val: usize) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8.val);
        Sq { val }
    }
}

impl From<u64> for Sq {
    fn from(val: u64) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8.val);
        Sq { val }
    }
}

impl From<u32> for Sq {
    fn from(val: u32) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8.val);
        Sq { val }
    }
}

impl From<u16> for Sq {
    fn from(val: u16) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8.val);
        Sq { val }
    }
}

impl From<u8> for Sq {
    fn from(val: u8) -> Sq {
        assert!(val <= H8.val);
        Sq { val }
    }
}

impl From<Sq> for usize {
    fn from(sq: Sq) -> usize {
        sq.val as usize
    }
}

impl From<Sq> for u64 {
    fn from(sq: Sq) -> u64 {
        sq.val as u64
    }
}

impl From<Sq> for u32 {
    fn from(sq: Sq) -> u32 {
        sq.val as u32
    }
}

impl From<Sq> for u16 {
    fn from(sq: Sq) -> u16 {
        sq.val as u16
    }
}

impl From<Sq> for u8 {
    fn from(sq: Sq) -> u8 {
        sq.val as u8
    }
}

impl Shl<Sq> for u64 {
    type Output = u64;
    #[inline]
    fn shl(self, sq: Sq) -> u64 {
        self << sq.val
    }
}

/// Constants for the squares of the board.
pub const A1: Sq = Sq { val: 0u8 };
pub const B1: Sq = Sq { val: 1u8 };
pub const C1: Sq = Sq { val: 2u8 };
pub const D1: Sq = Sq { val: 3u8 };
pub const E1: Sq = Sq { val: 4u8 };
pub const F1: Sq = Sq { val: 5u8 };
pub const G1: Sq = Sq { val: 6u8 };
pub const H1: Sq = Sq { val: 7u8 };
pub const A2: Sq = Sq { val: 8u8 };
pub const B2: Sq = Sq { val: 9u8 };
pub const C2: Sq = Sq { val: 10u8 };
pub const D2: Sq = Sq { val: 11u8 };
pub const E2: Sq = Sq { val: 12u8 };
pub const F2: Sq = Sq { val: 13u8 };
pub const G2: Sq = Sq { val: 14u8 };
pub const H2: Sq = Sq { val: 15u8 };
pub const A3: Sq = Sq { val: 16u8 };
pub const B3: Sq = Sq { val: 17u8 };
pub const C3: Sq = Sq { val: 18u8 };
pub const D3: Sq = Sq { val: 19u8 };
pub const E3: Sq = Sq { val: 20u8 };
pub const F3: Sq = Sq { val: 21u8 };
pub const G3: Sq = Sq { val: 22u8 };
pub const H3: Sq = Sq { val: 23u8 };
pub const A4: Sq = Sq { val: 24u8 };
pub const B4: Sq = Sq { val: 25u8 };
pub const C4: Sq = Sq { val: 26u8 };
pub const D4: Sq = Sq { val: 27u8 };
pub const E4: Sq = Sq { val: 28u8 };
pub const F4: Sq = Sq { val: 29u8 };
pub const G4: Sq = Sq { val: 30u8 };
pub const H4: Sq = Sq { val: 31u8 };
pub const A5: Sq = Sq { val: 32u8 };
pub const B5: Sq = Sq { val: 33u8 };
pub const C5: Sq = Sq { val: 34u8 };
pub const D5: Sq = Sq { val: 35u8 };
pub const E5: Sq = Sq { val: 36u8 };
pub const F5: Sq = Sq { val: 37u8 };
pub const G5: Sq = Sq { val: 38u8 };
pub const H5: Sq = Sq { val: 39u8 };
pub const A6: Sq = Sq { val: 40u8 };
pub const B6: Sq = Sq { val: 41u8 };
pub const C6: Sq = Sq { val: 42u8 };
pub const D6: Sq = Sq { val: 43u8 };
pub const E6: Sq = Sq { val: 44u8 };
pub const F6: Sq = Sq { val: 45u8 };
pub const G6: Sq = Sq { val: 46u8 };
pub const H6: Sq = Sq { val: 47u8 };
pub const A7: Sq = Sq { val: 48u8 };
pub const B7: Sq = Sq { val: 49u8 };
pub const C7: Sq = Sq { val: 50u8 };
pub const D7: Sq = Sq { val: 51u8 };
pub const E7: Sq = Sq { val: 52u8 };
pub const F7: Sq = Sq { val: 53u8 };
pub const G7: Sq = Sq { val: 54u8 };
pub const H7: Sq = Sq { val: 55u8 };
pub const A8: Sq = Sq { val: 56u8 };
pub const B8: Sq = Sq { val: 57u8 };
pub const C8: Sq = Sq { val: 58u8 };
pub const D8: Sq = Sq { val: 59u8 };
pub const E8: Sq = Sq { val: 60u8 };
pub const F8: Sq = Sq { val: 61u8 };
pub const G8: Sq = Sq { val: 62u8 };
pub const H8: Sq = Sq { val: 63u8 };

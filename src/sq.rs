use std::convert::From;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Sq {
    val: u8,
}

impl Debug for Sq {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        // It's safe to unwrap because constructing invalid Square results in panic.
        write!(f, "{}", sqstr(self.val).unwrap())
    }
}

impl From<u64> for Sq {
    fn from(val: u64) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8);
        Sq { val }
    }
}

impl From<u32> for Sq {
    fn from(val: u32) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8);
        Sq { val }
    }
}

impl From<u16> for Sq {
    fn from(val: u16) -> Sq {
        let val = (0xff & val) as u8;
        assert!(val <= H8);
        Sq { val }
    }
}

impl From<u8> for Sq {
    fn from(val: u8) -> Sq {
        assert!(val <= H8);
        Sq { val }
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

pub fn sqstr(square: u8) -> Result<&'static str, String> {
    match square {
        A1 => Ok("A1"),
        B1 => Ok("B1"),
        C1 => Ok("C1"),
        D1 => Ok("D1"),
        E1 => Ok("E1"),
        F1 => Ok("F1"),
        G1 => Ok("G1"),
        H1 => Ok("H1"),
        A2 => Ok("A2"),
        B2 => Ok("B2"),
        C2 => Ok("C2"),
        D2 => Ok("D2"),
        E2 => Ok("E2"),
        F2 => Ok("F2"),
        G2 => Ok("G2"),
        H2 => Ok("H2"),
        A3 => Ok("A3"),
        B3 => Ok("B3"),
        C3 => Ok("C3"),
        D3 => Ok("D3"),
        E3 => Ok("E3"),
        F3 => Ok("F3"),
        G3 => Ok("G3"),
        H3 => Ok("H3"),
        A4 => Ok("A4"),
        B4 => Ok("B4"),
        C4 => Ok("C4"),
        D4 => Ok("D4"),
        E4 => Ok("E4"),
        F4 => Ok("F4"),
        G4 => Ok("G4"),
        H4 => Ok("H4"),
        A5 => Ok("A5"),
        B5 => Ok("B5"),
        C5 => Ok("C5"),
        D5 => Ok("D5"),
        E5 => Ok("E5"),
        F5 => Ok("F5"),
        G5 => Ok("G5"),
        H5 => Ok("H5"),
        A6 => Ok("A6"),
        B6 => Ok("B6"),
        C6 => Ok("C6"),
        D6 => Ok("D6"),
        E6 => Ok("E6"),
        F6 => Ok("F6"),
        G6 => Ok("G6"),
        H6 => Ok("H6"),
        A7 => Ok("A7"),
        B7 => Ok("B7"),
        C7 => Ok("C7"),
        D7 => Ok("D7"),
        E7 => Ok("E7"),
        F7 => Ok("F7"),
        G7 => Ok("G7"),
        H7 => Ok("H7"),
        A8 => Ok("A8"),
        B8 => Ok("B8"),
        C8 => Ok("C8"),
        D8 => Ok("D8"),
        E8 => Ok("E8"),
        F8 => Ok("F8"),
        G8 => Ok("G8"),
        H8 => Ok("H8"),
        _ => Err(format!("{} is not a valid square number", square)),
    }
}

/// Constants for the squares of the board.
pub const A1: u8 = 0;
pub const B1: u8 = 1;
pub const C1: u8 = 2;
pub const D1: u8 = 3;
pub const E1: u8 = 4;
pub const F1: u8 = 5;
pub const G1: u8 = 6;
pub const H1: u8 = 7;
pub const A2: u8 = 8;
pub const B2: u8 = 9;
pub const C2: u8 = 10;
pub const D2: u8 = 11;
pub const E2: u8 = 12;
pub const F2: u8 = 13;
pub const G2: u8 = 14;
pub const H2: u8 = 15;
pub const A3: u8 = 16;
pub const B3: u8 = 17;
pub const C3: u8 = 18;
pub const D3: u8 = 19;
pub const E3: u8 = 20;
pub const F3: u8 = 21;
pub const G3: u8 = 22;
pub const H3: u8 = 23;
pub const A4: u8 = 24;
pub const B4: u8 = 25;
pub const C4: u8 = 26;
pub const D4: u8 = 27;
pub const E4: u8 = 28;
pub const F4: u8 = 29;
pub const G4: u8 = 30;
pub const H4: u8 = 31;
pub const A5: u8 = 32;
pub const B5: u8 = 33;
pub const C5: u8 = 34;
pub const D5: u8 = 35;
pub const E5: u8 = 36;
pub const F5: u8 = 37;
pub const G5: u8 = 38;
pub const H5: u8 = 39;
pub const A6: u8 = 40;
pub const B6: u8 = 41;
pub const C6: u8 = 42;
pub const D6: u8 = 43;
pub const E6: u8 = 44;
pub const F6: u8 = 45;
pub const G6: u8 = 46;
pub const H6: u8 = 47;
pub const A7: u8 = 48;
pub const B7: u8 = 49;
pub const C7: u8 = 50;
pub const D7: u8 = 51;
pub const E7: u8 = 52;
pub const F7: u8 = 53;
pub const G7: u8 = 54;
pub const H7: u8 = 55;
pub const A8: u8 = 56;
pub const B8: u8 = 57;
pub const C8: u8 = 58;
pub const D8: u8 = 59;
pub const E8: u8 = 60;
pub const F8: u8 = 61;
pub const G8: u8 = 62;
pub const H8: u8 = 63;

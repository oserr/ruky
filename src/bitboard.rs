use crate::sq::{self, Sq};
use num::{PrimInt, Unsigned};
use std::convert::{From, Into};
use std::fmt::{self, Debug, Formatter};
use std::ops::{BitAnd, BitOr, BitOrAssign, Mul, Not, Shl, Shr, ShrAssign};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct BitBoard {
    /// The raw bits used to represent the BitBoard.
    bits: u64,
}

impl Not for BitBoard {
    type Output = Self;
    fn not(self) -> BitBoard {
        BitBoard::from(!self.bits)
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard::from(self.bits & rhs.bits)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;
    fn bitor(self, rhs: BitBoard) -> BitBoard {
        BitBoard::from(self.bits | rhs.bits)
    }
}

impl BitOrAssign<Sq> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Sq) {
        self.set_bit(rhs.into());
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: BitBoard) {
        self.bits |= rhs.bits;
    }
}

impl<T> Shr<T> for BitBoard
where
    T: PrimInt,
    u64: Shr<T, Output = u64>,
{
    type Output = BitBoard;
    fn shr(self, rhs: T) -> BitBoard {
        BitBoard::from(self.bits >> rhs)
    }
}

impl<T> ShrAssign<T> for BitBoard
where
    T: PrimInt,
    u64: Shr<T, Output = u64>,
    u64: ShrAssign<T>,
{
    fn shr_assign(&mut self, rhs: T) {
        self.bits >>= rhs;
    }
}

impl<T> Shl<T> for BitBoard
where
    T: PrimInt,
    u64: Shl<T, Output = u64>,
{
    type Output = BitBoard;
    fn shl(self, rhs: T) -> BitBoard {
        BitBoard::from(self.bits << rhs)
    }
}

impl<T> Mul<T> for BitBoard
where
    T: Unsigned,
    T: Into<u64>,
    u64: Mul<T, Output = u64>,
{
    type Output = BitBoard;
    fn mul(self, rhs: T) -> BitBoard {
        BitBoard::from(self.bits.wrapping_mul(rhs.into()))
    }
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "BitBoard")?;
        f.debug_list().entries(self.sq_iter()).finish()
    }
}

impl From<u64> for BitBoard {
    fn from(bits: u64) -> BitBoard {
        BitBoard { bits }
    }
}

impl<T> From<BitBoard> for Vec<T>
where
    T: Unsigned,
    T: From<Sq>,
{
    fn from(bits: BitBoard) -> Vec<T> {
        bits.sq_iter().map(|s| s.into()).collect()
    }
}

impl<T> From<&[T]> for BitBoard
where
    T: PrimInt,
    u64: Shl<T, Output = u64>,
{
    fn from(numbers: &[T]) -> BitBoard {
        let bits = numbers.iter().fold(0u64, |bits, n| bits | (1u64 << *n));
        BitBoard { bits }
    }
}

impl<T, const N: usize> From<&[T; N]> for BitBoard
where
    T: PrimInt,
    u64: Shl<T, Output = u64>,
{
    fn from(numbers: &[T; N]) -> BitBoard {
        BitBoard::from(&numbers[..])
    }
}

impl<T> From<&Vec<T>> for BitBoard
where
    T: PrimInt,
    u64: Shl<T, Output = u64>,
{
    fn from(numbers: &Vec<T>) -> BitBoard {
        BitBoard::from(&numbers[..])
    }
}

impl From<Sq> for BitBoard {
    fn from(sq: Sq) -> BitBoard {
        BitBoard::from(1u64 << sq)
    }
}

impl From<&[Sq]> for BitBoard {
    fn from(squares: &[Sq]) -> BitBoard {
        squares
            .iter()
            .fold(BitBoard::new(), |mut b, s| *b.set_bit(*s))
    }
}

impl<const N: usize> From<&[Sq; N]> for BitBoard {
    fn from(squares: &[Sq; N]) -> BitBoard {
        BitBoard::from(&squares[..])
    }
}

impl From<&Vec<Sq>> for BitBoard {
    fn from(squares: &Vec<Sq>) -> BitBoard {
        BitBoard::from(&squares[..])
    }
}

impl From<BitBoard> for Vec<Sq> {
    fn from(bitboard: BitBoard) -> Vec<Sq> {
        bitboard.sq_iter().collect()
    }
}

impl BitBoard {
    /// Creates a new Bitboard instance with all bits cleared.
    pub fn new() -> BitBoard {
        BitBoard::default()
    }

    /// Creates a new Bitboard instance with all bits cleared.
    pub fn u64(&self) -> u64 {
        self.bits
    }

    /// Returns true if any bits are set.
    pub fn any(&self) -> bool {
        self.bits != 0
    }

    /// Returns true if no bits are set, or false otherwise.
    pub fn none(&self) -> bool {
        !self.any()
    }

    /// Clears all the bits.
    pub fn clear(&mut self) -> &mut Self {
        self.bits = 0;
        self
    }

    /// Checks if a given bit is set. Index is zero based.
    pub fn has_bit(&self, sq: Sq) -> bool {
        self.bits & (1u64 << sq) != 0
    }

    /// Clears a given bit if set. Index is zero based.
    pub fn clear_bit(&mut self, sq: Sq) -> &mut Self {
        self.bits &= !(1u64 << sq);
        self
    }

    /// Clears a given bit if set, otherwise returns an error. Index is zero
    /// based.
    pub fn clear_bit_or(&mut self, sq: Sq) -> Result<&mut Self, BitErr> {
        if !self.has_bit(sq) {
            return Err(BitErr::IsNotSet(sq));
        }
        self.bits &= !(1u64 << sq);
        Ok(self)
    }

    /// Sets a given bit.
    pub fn set_bit(&mut self, sq: Sq) -> &mut Self {
        self.bits |= 1u64 << sq;
        self
    }

    /// Sets a given bit if the bit is not set, otherwise returns an error.
    pub fn set_bit_or(&mut self, sq: Sq) -> Result<&mut Self, BitErr> {
        if self.has_bit(sq) {
            return Err(BitErr::IsSetAlready(sq));
        }
        self.bits |= 1u64 << sq;
        Ok(self)
    }

    /// Updates a bit by setting to zero in |from| and setting it in |to|. If
    /// |from| is not set or |to| is already set, then we return an error.
    pub fn update_bit(&mut self, from: Sq, to: Sq) -> Result<&mut Self, BitErr> {
        if !self.has_bit(from) {
            Err(BitErr::FromIsNotSet(from))
        } else if self.has_bit(to) {
            Err(BitErr::ToIsSetAlready(to))
        } else {
            Ok(self.clear_bit(from).set_bit(to))
        }
    }

    /// Updates a bit by setting to zero in |from| and setting it in |to|. If
    /// |from| is not set or |to| is already set, then we return an error.
    pub fn set_bits(&mut self, other_bits: u64) -> &mut Self {
        self.bits = other_bits;
        self
    }

    /// Returns the number of bits set.
    pub fn count(&self) -> u8 {
        self.bits.count_ones() as u8
    }

    /// Returns true if only bit is set, false otherwise.
    pub fn is_single(&self) -> bool {
        self.bits.is_power_of_two()
    }

    /// Returns the square of the first bit set. If no bit is set, returns None.
    pub fn first_bit(&self) -> Option<Sq> {
        if self.any() {
            Some(self.bits.trailing_zeros().into())
        } else {
            None
        }
    }

    /// Returns the index of the first bit set and clears it from the bitboard
    /// if any bits are set, otherwise returns None.
    pub fn take_first(&mut self) -> Option<Sq> {
        let s = self.first_bit();
        if s.is_some() {
            self.bits &= self.bits - 1;
        }
        s
    }

    // Returns an iterator over the squares where the bits are set.
    pub fn sq_iter(&self) -> impl Iterator<Item = Sq> {
        SquareIter::from(*self)
    }

    // Returns an iterator over the squares where the bits are set.
    pub fn sq_bit_iter(&self) -> impl Iterator<Item = (Sq, BitBoard)> {
        SquareBitIter::from(*self)
    }

    // Returns an iterator over the squares where the bits are set.
    pub fn to_vec<T: Unsigned + From<Sq>>(&self) -> Vec<T> {
        self.clone().into()
    }

    // Returns a BitBoard with with the bits set to represent the squares where a
    // king would be able to move assuming there are no blockers.
    pub fn king_moves(&self) -> BitBoard {
        let mut k = *self;
        let left = k.sh_file_a();
        let right = k.sh_file_h();
        k |= left | right;
        let down = k.sh_rank_1();
        let up = k.sh_rank_8();
        (k | down | up) & self.not()
    }

    // Returns a BitBoard with with the bits set to represent the squares where a
    // knight would be able to move assuming there are no blockers.
    pub fn knight_moves(&self) -> BitBoard {
        // One square up, two right.
        let bits1 = (*self & !FILE_G & !FILE_H) << 10;
        // Two square up, one right.
        let bits2 = (*self & !FILE_H) << 17;
        // Two square up, one left.
        let bits3 = (*self & !FILE_A) << 15;
        // One square up, two left.
        let bits4 = (*self & !FILE_A & !FILE_B) << 6;
        // One square down, two left.
        let bits5 = (*self & !FILE_A & !FILE_B) >> 10;
        // Two square down, one left.
        let bits6 = (*self & !FILE_A) >> 17;
        // Two square down, one right.
        let bits7 = (*self & !FILE_H) >> 15;
        // One square down, two right.
        let bits8 = (*self & !FILE_G & !FILE_H) >> 6;

        bits1 | bits2 | bits3 | bits4 | bits5 | bits6 | bits7 | bits8
    }

    // Returns a pair of bitboards in the form of (forward moves, attacks) for white
    // pawns.
    pub fn wp_moves(&self, empty: BitBoard) -> (BitBoard, BitBoard) {
        let forward = self.wp_single(empty) | self.wp_double(empty);
        let attacks = self.wp_left() | self.wp_right();
        (forward, attacks)
    }

    // Returns a BitBoard with with the bits set to represent the squares for after
    // moving white pawns forward one square toward rank 8.
    #[inline]
    pub fn wp_single(&self, empty: BitBoard) -> BitBoard {
        self.sh_rank_8() & empty
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving white pawns forward two squares toward rank 8. Pawns can only do
    // this on their first move when moving from the second to the fourth rank.
    pub fn wp_double(&self, empty: BitBoard) -> BitBoard {
        (self.sh_rank_8() & empty).sh_rank_8() & empty & RANK_4
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving a white pawn across the left diagonal for a capture.
    #[inline]
    pub fn wp_left(&self) -> BitBoard {
        (*self << 7) & !FILE_H
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving a white pawn across the right diagonal for a capture.
    #[inline]
    pub fn wp_right(&self) -> BitBoard {
        (*self << 9) & !FILE_A
    }

    // Returns a pair of bitboards in the form of (forward moves, attacks) for black
    // pawns.
    pub fn bp_moves(&self, empty: BitBoard) -> (BitBoard, BitBoard) {
        let forward = self.bp_single(empty) | self.bp_double(empty);
        let attacks = self.bp_left() | self.bp_right();
        (forward, attacks)
    }

    // Returns a BitBoard with with the bits set to represent the squares for after
    // moving black pawns forward one square toward rank 1.
    #[inline]
    pub fn bp_single(&self, empty: BitBoard) -> BitBoard {
        self.sh_rank_1() & empty
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving black pawns forward two squares toward rank 1. Pawns can only do
    // this on their first move when moving from the 7th to the 5th rank.
    pub fn bp_double(&self, empty: BitBoard) -> BitBoard {
        (self.sh_rank_1() & empty).sh_rank_1() & empty & RANK_5
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving a black pawn across the left diagonal for a capture. Note that
    // left here is from the perspective of black, toward file H.
    #[inline]
    pub fn bp_left(&self) -> BitBoard {
        (*self >> 7) & !FILE_A
    }

    // Returns a BitBoard with with the bits set to represent the squares after
    // moving a black pawn across the right diagonal, toward file A, for a
    // capture.
    #[inline]
    pub fn bp_right(&self) -> BitBoard {
        (*self >> 9) & !FILE_H
    }

    // Shifts bits toward file A by one bit without wrapping bits already on file A.
    #[inline]
    pub fn sh_file_a(&self) -> BitBoard {
        (*self & !FILE_A) >> 1
    }

    // Shifts bits toward file H by one bit without wrapping bits already on file H.
    #[inline]
    pub fn sh_file_h(&self) -> BitBoard {
        (*self & !FILE_H) << 1
    }

    // Shifts bits toward rank 1 by one row.
    #[inline]
    pub fn sh_rank_1(&self) -> BitBoard {
        *self >> 8
    }

    // Shifts bits toward rank 8 by one row.
    #[inline]
    pub fn sh_rank_8(&self) -> BitBoard {
        *self << 8
    }
}

// A struct to iterate over the set bits of a BitBoard as if they were squares.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SquareIter {
    bits: BitBoard,
}

// Converts a BitBoard to a SquareIter.
impl From<BitBoard> for SquareIter {
    fn from(bits: BitBoard) -> SquareIter {
        SquareIter { bits }
    }
}

// Mechanics of iterating over BitBoard as squares.
impl Iterator for SquareIter {
    type Item = Sq;
    fn next(&mut self) -> Option<Sq> {
        self.bits.take_first().map(Sq::from)
    }
}

// A struct to iterate over the set bits of a BitBoard as pairs of the form (Sq,
// BitBoard), where the first item represents the square of a bit that is set,
// and the second item is the one-bit BitBoard for that square.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SquareBitIter {
    it: SquareIter,
}

// Mechanics of iterating over BitBoard to get the pair (Sq, BitBoard).
impl Iterator for SquareBitIter {
    type Item = (Sq, BitBoard);
    fn next(&mut self) -> Option<(Sq, BitBoard)> {
        self.it.next().map(|sq| (sq, BitBoard::from(sq)))
    }
}

// Converts a BitBoard to a SquareBitIter.
impl From<BitBoard> for SquareBitIter {
    fn from(bits: BitBoard) -> SquareBitIter {
        SquareBitIter {
            it: SquareIter::from(bits),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq)]
pub enum BitErr {
    #[error("bit {0:?} is not set")]
    IsNotSet(Sq),
    #[error("bit {0:?} is already set")]
    IsSetAlready(Sq),
    #[error("from bit {0:?} is not set")]
    FromIsNotSet(Sq),
    #[error("to bit {0:?} is already set")]
    ToIsSetAlready(Sq),
}

// Files/columns.
pub const FILE_A: BitBoard = BitBoard {
    bits: 0x0101010101010101u64,
};
pub const FILE_B: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 1,
};
pub const FILE_C: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 2,
};
pub const FILE_D: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 3,
};
pub const FILE_E: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 4,
};
pub const FILE_F: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 5,
};
pub const FILE_G: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 6,
};
pub const FILE_H: BitBoard = BitBoard {
    bits: 0x0101010101010101u64 << 7,
};

// Ranks/rows.
pub const RANK_1: BitBoard = BitBoard { bits: 0xffu64 };
pub const RANK_2: BitBoard = BitBoard { bits: 0xffu64 << 8 };
pub const RANK_3: BitBoard = BitBoard {
    bits: 0xffu64 << 16,
};
pub const RANK_4: BitBoard = BitBoard {
    bits: 0xffu64 << 24,
};
pub const RANK_5: BitBoard = BitBoard {
    bits: 0xffu64 << 32,
};
pub const RANK_6: BitBoard = BitBoard {
    bits: 0xffu64 << 40,
};
pub const RANK_7: BitBoard = BitBoard {
    bits: 0xffu64 << 48,
};
pub const RANK_8: BitBoard = BitBoard {
    bits: 0xffu64 << 56,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_default() {
        assert_eq!(BitBoard::default(), BitBoard::new());
    }

    #[test]
    fn from_one_has_first_bit_set() {
        let b = BitBoard::from(1);
        assert!(b.has_bit(0u8.into()));
    }

    #[test]
    fn from_two_has_second_bit_set() {
        let b = BitBoard::from(2);
        assert!(b.has_bit(1u8.into()));
    }

    #[test]
    fn from_slice() {
        assert_eq!(BitBoard::from(&[0, 1, 2]), BitBoard::from(0b111));
    }

    #[test]
    fn clear_first() {
        let mut b = BitBoard::from(1);
        assert!(!b.clear_bit(0u8.into()).has_bit(0u8.into()));
    }

    #[test]
    fn clear_none() {
        let mut b = BitBoard::from(1);
        assert!(b.clear_bit(1u8.into()).has_bit(0u8.into()));
    }

    #[test]
    fn set_first_two_bits() {
        let mut b = BitBoard::new();
        b.set_bit(0u8.into()).set_bit(1u8.into());
        assert!(b.has_bit(0u8.into()));
        assert!(b.has_bit(1u8.into()));
    }

    #[test]
    fn count() {
        let mut b = BitBoard::new();
        b.set_bit(0u8.into());
        assert_eq!(b.count(), 1);
        b.set_bit(1u8.into());
        assert_eq!(b.count(), 2);
        b.set_bit(2u8.into());
        assert_eq!(b.count(), 3);
    }

    #[test]
    fn update_bit() {
        let five = Sq::from(5u8);
        let seven = Sq::from(7u8);
        let eight = Sq::from(8u8);
        let mut b = BitBoard::new();
        assert_eq!(b.update_bit(five, seven), Err(BitErr::FromIsNotSet(five)));
        b.set_bit(five);
        b.set_bit(seven);
        assert_eq!(
            b.update_bit(five, seven),
            Err(BitErr::ToIsSetAlready(seven))
        );
        assert_eq!(b.update_bit(five, eight), Ok(&mut BitBoard::from(&[7, 8])));
    }

    #[test]
    fn set_bits() {
        let mut b = BitBoard::from(&[1, 2, 3, 4]);
        b.set_bits(0b110);
        assert_eq!(b, BitBoard::from(&[1, 2]));
    }

    #[test]
    fn is_single() {
        let mut b = BitBoard::from(&[1, 2, 3, 4]);
        assert!(!b.is_single());

        b.set_bits(0b10000);
        assert!(b.is_single());
    }

    #[test]
    fn first_bit() {
        let mut b = BitBoard::new();
        assert_eq!(b.first_bit(), None);

        for i in 0u8..=63 {
            let n = 1u64 << i;
            b.set_bits(n);
            assert_eq!(b.first_bit(), Some(i.into()));
        }
    }

    #[test]
    fn take_first_bit() {
        let mut b = BitBoard::from(1u64 << 10);
        assert_eq!(b.take_first(), Some(10u8.into()));
        assert_eq!(b.take_first(), None);
        assert!(!b.any());

        b.set_bit(30u8.into()).set_bit(40u8.into());
        assert_eq!(b.take_first(), Some(30u8.into()));
        assert_eq!(b.take_first(), Some(40u8.into()));
        assert_eq!(b.take_first(), None);
        assert!(!b.any());
    }

    #[test]
    fn iterate_over_squares() {
        let b = BitBoard::from(&[0, 55, 3, 60, 23, 10, 11, 35]);
        assert_eq!(
            b.sq_iter().map(|s| s.into()).collect::<Vec<u8>>(),
            vec![0, 3, 10, 11, 23, 35, 55, 60]
        );
    }

    #[test]
    fn into_from_vec() {
        let v = vec![3, 6, 9];
        let b = BitBoard::from(&v);
        assert_eq!(b.to_vec::<u8>(), v);
    }

    #[test]
    fn king_moves_from_a1() {
        assert_eq!(
            BitBoard::from(sq::A1).king_moves(),
            BitBoard::from(&[sq::B1, sq::A2, sq::B2])
        );
    }

    #[test]
    fn king_moves_from_e1() {
        assert_eq!(
            BitBoard::from(sq::E1).king_moves(),
            BitBoard::from(&[sq::D1, sq::D2, sq::E2, sq::F1, sq::F2])
        );
    }

    #[test]
    fn king_moves_from_h5() {
        assert_eq!(
            BitBoard::from(sq::H5).king_moves(),
            BitBoard::from(&[sq::H4, sq::G4, sq::G5, sq::G6, sq::H6])
        );
    }

    #[test]
    fn king_moves_from_e5() {
        assert_eq!(
            BitBoard::from(sq::E5).king_moves(),
            BitBoard::from(&[
                sq::D4,
                sq::D5,
                sq::D6,
                sq::E4,
                sq::E6,
                sq::F4,
                sq::F5,
                sq::F6
            ])
        );
    }

    #[test]
    fn king_moves_from_g8() {
        assert_eq!(
            BitBoard::from(sq::G8).king_moves(),
            BitBoard::from(&[sq::F8, sq::H8, sq::F7, sq::G7, sq::H7])
        );
    }

    #[test]
    fn knight_moves_from_b1() {
        assert_eq!(
            BitBoard::from(sq::B1).knight_moves(),
            BitBoard::from(&[sq::A3, sq::C3, sq::D2])
        );
    }

    #[test]
    fn knight_moves_from_d4() {
        assert_eq!(
            BitBoard::from(sq::D4).knight_moves(),
            BitBoard::from(&[
                sq::C2,
                sq::B3,
                sq::B5,
                sq::C6,
                sq::E6,
                sq::F5,
                sq::F3,
                sq::E2
            ])
        );
    }

    #[test]
    fn knight_moves_from_g5() {
        assert_eq!(
            BitBoard::from(sq::G5).knight_moves(),
            BitBoard::from(&[sq::F3, sq::E4, sq::E6, sq::F7, sq::H7, sq::H3])
        );
    }

    #[test]
    fn knight_moves_from_a8() {
        assert_eq!(
            BitBoard::from(sq::A8).knight_moves(),
            BitBoard::from(&[sq::B6, sq::C7])
        );
    }

    #[test]
    fn wp_single() {
        assert_eq!(
            BitBoard::from(&[sq::A2, sq::C6, sq::D5, sq::G7, sq::H8]).wp_single(!BitBoard::new()),
            BitBoard::from(&[sq::A3, sq::C7, sq::D6, sq::G8])
        );
    }

    #[test]
    fn wp_double() {
        assert_eq!(
            BitBoard::from(&[sq::A2, sq::C6, sq::D5, sq::G2, sq::H2]).wp_double(!BitBoard::new()),
            BitBoard::from(&[sq::A4, sq::G4, sq::H4])
        );
    }

    #[test]
    fn wp_left() {
        assert_eq!(
            BitBoard::from(&[sq::A2, sq::C6, sq::G2, sq::H4]).wp_left(),
            BitBoard::from(&[sq::B7, sq::F3, sq::G5])
        );
    }

    #[test]
    fn wp_right() {
        assert_eq!(
            BitBoard::from(&[sq::A2, sq::C6, sq::G2, sq::H4]).wp_right(),
            BitBoard::from(&[sq::B3, sq::D7, sq::H3])
        );
    }

    #[test]
    fn bp_single() {
        assert_eq!(
            BitBoard::from(&[sq::B7, sq::C6, sq::D7, sq::G8, sq::H3]).bp_single(!BitBoard::new()),
            BitBoard::from(&[sq::B6, sq::C5, sq::D6, sq::G7, sq::H2])
        );
    }

    #[test]
    fn bp_double() {
        assert_eq!(
            BitBoard::from(&[sq::B7, sq::C6, sq::D7, sq::G8, sq::H3]).bp_double(!BitBoard::new()),
            BitBoard::from(&[sq::B5, sq::D5])
        );
    }

    #[test]
    fn bp_left() {
        assert_eq!(
            BitBoard::from(&[sq::B7, sq::C6, sq::D7, sq::H3]).bp_left(),
            BitBoard::from(&[sq::C6, sq::D5, sq::E6])
        );
    }

    #[test]
    fn bp_right() {
        assert_eq!(
            BitBoard::from(&[sq::A7, sq::C6, sq::D7, sq::H3]).bp_right(),
            BitBoard::from(&[sq::B5, sq::C6, sq::G2])
        );
    }
}

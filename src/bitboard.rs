use crate::sq::Sq;
use num::{PrimInt, Unsigned};
use std::convert::{From, Into};
use std::fmt::{self, Debug, Formatter};
use std::ops::{BitAnd, BitOrAssign, Mul, Shl, Shr};

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct BitBoard {
    /// The raw bits used to represent the BitBoard.
    bits: u64,
}

impl BitAnd for BitBoard {
    type Output = BitBoard;
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard::from(self.bits & rhs.bits)
    }
}

impl BitOrAssign<Sq> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Sq) {
        self.set_bit(rhs.into());
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

    /// Clears a given bit if set, otherwise returns an error. Index is zero based.
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

    /// Updates a bit by setting to zero in |from| and setting it in |to|. If |from| is not set or
    /// |to| is already set, then we return an error.
    pub fn update_bit(&mut self, from: Sq, to: Sq) -> Result<&mut Self, BitErr> {
        if !self.has_bit(from) {
            Err(BitErr::FromIsNotSet(from))
        } else if self.has_bit(to) {
            Err(BitErr::ToIsSetAlready(to))
        } else {
            Ok(self.clear_bit(from).set_bit(to))
        }
    }

    /// Updates a bit by setting to zero in |from| and setting it in |to|. If |from| is not set or
    /// |to| is already set, then we return an error.
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

    /// Returns the index of the first bit set. If no bit is set, returns 64.
    pub fn first_bit(&self) -> u8 {
        self.bits.trailing_zeros() as u8
    }

    /// Returns the index of the first bit set and clears it from the bitboard if any bits are set,
    /// otherwise returns None.
    pub fn take_first(&mut self) -> Option<u8> {
        if !self.any() {
            return None;
        }
        let i = self.first_bit();
        self.bits &= self.bits - 1;
        Some(i)
    }

    // Returns an iterator over the squares where the bits are set.
    pub fn sq_iter(&self) -> impl Iterator<Item = Sq> {
        SquareIter::from(*self)
    }

    // Returns an iterator over the squares where the bits are set.
    pub fn to_vec<T: Unsigned + From<Sq>>(&self) -> Vec<T> {
        self.clone().into()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SquareIter {
    bits: BitBoard,
}

impl From<BitBoard> for SquareIter {
    fn from(bits: BitBoard) -> SquareIter {
        SquareIter { bits }
    }
}

impl Iterator for SquareIter {
    type Item = Sq;
    fn next(&mut self) -> Option<Sq> {
        self.bits.take_first().map(Sq::from)
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
        assert!(b.has_bit(0.into()));
    }

    #[test]
    fn from_two_has_second_bit_set() {
        let b = BitBoard::from(2);
        assert!(b.has_bit(1.into()));
    }

    #[test]
    fn from_slice() {
        assert_eq!(BitBoard::from(&[0, 1, 2]), BitBoard::from(0b111));
    }

    #[test]
    fn clear_first() {
        let mut b = BitBoard::from(1);
        assert!(!b.clear_bit(0.into()).has_bit(0.into()));
    }

    #[test]
    fn clear_none() {
        let mut b = BitBoard::from(1);
        assert!(b.clear_bit(1.into()).has_bit(0.into()));
    }

    #[test]
    fn set_first_two_bits() {
        let mut b = BitBoard::new();
        b.set_bit(0).set_bit(1);
        assert!(b.has_bit(0.into()));
        assert!(b.has_bit(1.into()));
    }

    #[test]
    fn count() {
        let mut b = BitBoard::new();
        b.set_bit(0.into());
        assert_eq!(b.count(), 1);
        b.set_bit(1.into());
        assert_eq!(b.count(), 2);
        b.set_bit(2.into());
        assert_eq!(b.count(), 3);
    }

    #[test]
    fn update_bit() {
        let five = Square::from(5);
        let seven = Square::from(7);
        let eight = Square::from(8);
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
        assert_eq!(b.first_bit(), 64);

        for i in 0..=63 {
            let n = 1u64 << i;
            b.set_bits(n);
            assert_eq!(b.first_bit(), i);
        }
    }

    #[test]
    fn take_first_bit() {
        let mut b = BitBoard::from(1u64 << 10);
        assert_eq!(b.take_first(), Some(10));
        assert_eq!(b.take_first(), None);
        assert!(!b.any());

        b.set_bit(30.into()).set_bit(40.into());
        assert_eq!(b.take_first(), Some(30));
        assert_eq!(b.take_first(), Some(40));
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
}

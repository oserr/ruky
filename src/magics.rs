use crate::bitboard::BitBoard;
use crate::sq::Sq;

/// Computes a BitBoard with the full bishop mask for a given square, but ignores the outer edge
/// squares and the current square. For example, given square 0, i.e. square a1, it returns a
/// BitBoard with bits set at (b2, c3, d4, e5, f6, g7).
pub fn get_full_bmask(square: u32) -> BitBoard {
    let row = square / 8;
    let col = square % 8;
    let mut bits = BitBoard::new();

    // up-right
    for (r, c) in (row + 1..7).zip(col + 1..7) {
        bits.set_bit(r * 8 + c);
    }

    // down-right
    for (r, c) in (1..row).rev().zip(col + 1..7) {
        bits.set_bit(r * 8 + c);
    }

    // down-left
    for (r, c) in (1..row).rev().zip((1..col).rev()) {
        bits.set_bit(r * 8 + c);
    }

    // up-left
    for (r, c) in (row + 1..7).zip((1..col).rev()) {
        bits.set_bit(r * 8 + c);
    }

    return bits;
}

/// Computes a vector of 64 BitBoards, each representing the full bishop mask for a square.
pub fn get_full_bmasks() -> Vec<BitBoard> {
    (0..64).map(|i| get_full_bmask(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_full_bmask_for_a1() {
        let b = get_full_bmask(Sq::A1);
        assert_eq!(
            b,
            BitBoard::from(&[Sq::B2, Sq::C3, Sq::D4, Sq::E5, Sq::F6, Sq::G7])
        );
    }

    #[test]
    fn get_full_bmask_for_e5() {
        let b = get_full_bmask(Sq::E5);
        assert_eq!(
            b,
            BitBoard::from(&[
                Sq::B2,
                Sq::C3,
                Sq::D4,
                Sq::F6,
                Sq::G7,
                Sq::D6,
                Sq::C7,
                Sq::F4,
                Sq::G3
            ])
        );
    }

    #[test]
    fn get_full_bmasks_works() {
        let masks = get_full_bmasks();
        assert_eq!(masks.len(), 64);
        assert_eq!(
            masks[Sq::A1 as usize],
            BitBoard::from(&[Sq::B2, Sq::C3, Sq::D4, Sq::E5, Sq::F6, Sq::G7])
        );
        assert_eq!(
            masks[Sq::E5 as usize],
            BitBoard::from(&[
                Sq::B2,
                Sq::C3,
                Sq::D4,
                Sq::F6,
                Sq::G7,
                Sq::D6,
                Sq::C7,
                Sq::F4,
                Sq::G3
            ])
        );
    }
}

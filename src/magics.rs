use crate::bitboard::BitBoard;

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
    bits
}

/// Computes a vector of 64 BitBoards, each representing the full bishop mask for a square.
pub fn get_full_bmasks() -> Vec<BitBoard> {
    (0..64).map(|i| get_full_bmask(i)).collect()
}

/// Computes a BitBoard with the full rook mask for a given square, but ignores the outer edge
/// squares and the current square. For example, given square 0, i.e. square a1, it returns a
/// BitBoard with bits set at (a2, a3, a4, a5, a6, a7, b1, c1, d1, e1, f1, g1).
pub fn get_full_rmask(square: u32) -> BitBoard {
    let row = square / 8;
    let col = square % 8;
    let mut bits = BitBoard::new();
    // up
    for r in row + 1..7 {
        bits.set_bit(r * 8 + col);
    }
    // right
    for c in col + 1..7 {
        bits.set_bit(row * 8 + c);
    }
    // down
    for r in 1..row {
        bits.set_bit(r * 8 + col);
    }
    // left
    for c in 1..col {
        bits.set_bit(row * 8 + c);
    }
    bits
}

/// Computes a vector of 64 BitBoards, each representing the full rook mask for a square.
pub fn get_full_rmasks() -> Vec<BitBoard> {
    (0..64).map(|i| get_full_rmask(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sq::*;

    #[test]
    fn get_full_bmask_for_a1() {
        let b = get_full_bmask(A1);
        assert_eq!(b, BitBoard::from(&[B2, C3, D4, E5, F6, G7]));
    }

    #[test]
    fn get_full_bmask_for_e5() {
        let b = get_full_bmask(E5);
        assert_eq!(b, BitBoard::from(&[B2, C3, D4, F6, G7, D6, C7, F4, G3]));
    }

    #[test]
    fn get_full_bmasks_works() {
        let masks = get_full_bmasks();
        assert_eq!(masks.len(), 64);
        assert_eq!(
            masks[A1 as usize],
            BitBoard::from(&[B2, C3, D4, E5, F6, G7])
        );
        assert_eq!(
            masks[E5 as usize],
            BitBoard::from(&[B2, C3, D4, F6, G7, D6, C7, F4, G3])
        );
    }

    #[test]
    fn get_full_rmask_for_a1() {
        let b = get_full_rmask(A1);
        assert_eq!(
            b,
            BitBoard::from(&[A2, A3, A4, A5, A6, A7, B1, C1, D1, E1, F1, G1])
        );
    }

    #[test]
    fn get_full_rmask_for_e5() {
        let b = get_full_rmask(E5);
        assert_eq!(b, BitBoard::from(&[B5, C5, D5, F5, G5, E2, E3, E4, E6, E7]));
    }

    #[test]
    fn get_full_rmasks_works() {
        let masks = get_full_rmasks();
        assert_eq!(masks.len(), 64);
        assert_eq!(
            masks[A1 as usize],
            BitBoard::from(&[A2, A3, A4, A5, A6, A7, B1, C1, D1, E1, F1, G1])
        );
        assert_eq!(
            masks[E5 as usize],
            BitBoard::from(&[B5, C5, D5, F5, G5, E2, E3, E4, E6, E7])
        );
    }
}

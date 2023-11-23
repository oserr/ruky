use crate::bitboard::BitBoard;
use rand::RngCore;
use std::ops::Fn;

#[derive(Copy, Clone, Debug)]
pub enum MagicErr {
    InvalidSquare,
    NumBits,
    NotFound,
}

#[derive(Clone)]
pub struct Magic {
    pub attacks: Vec<BitBoard>,
    pub mask: BitBoard,
    pub magic: u64,
    pub rshift: u32,
}

impl Magic {
    fn get_hash(&self, blockers: BitBoard) -> usize {
        let b = blockers & self.mask;
        get_magic_hash(b, self.magic, self.rshift)
    }

    fn get_attacks(&self, blockers: BitBoard) -> BitBoard {
        let h = self.get_hash(blockers);
        self.attacks[h]
    }
}

pub trait Magics: AsRef<[Magic]> {
    fn attacks(&self, sq: u32, blockers: BitBoard) -> Option<BitBoard>;
    fn get(&self, sq: u32) -> Option<&Magic>;
}

pub struct MagicAttacks {
    magics: Vec<Magic>,
}

impl Magics for MagicAttacks {
    fn attacks(&self, sq: u32, blockers: BitBoard) -> Option<BitBoard> {
        let magic = self.magics.get(sq as usize)?;
        Some(magic.get_attacks(blockers))
    }

    fn get(&self, sq: u32) -> Option<&Magic> {
        self.magics.get(sq as usize)
    }
}

impl AsRef<[Magic]> for MagicAttacks {
    fn as_ref(&self) -> &[Magic] {
        self.magics.as_ref()
    }
}

pub fn compute_bmagics() -> Result<MagicAttacks, MagicErr> {
    let mut rand_iter = create_rand_iter();
    find_all_magics(&get_full_bmask, &get_battacks, rand_iter.by_ref())
}

pub fn compute_rmagics() -> Result<MagicAttacks, MagicErr> {
    let mut rand_iter = create_rand_iter();
    find_all_magics(&get_full_rmask, &get_rattacks, rand_iter.by_ref())
}

// Computes magics for all squares.
fn find_all_magics(
    mask_fn: &impl Fn(u32) -> BitBoard,
    attacks_fn: &impl Fn(u32, BitBoard) -> BitBoard,
    magic_iter: &mut impl Iterator<Item = u64>,
) -> Result<MagicAttacks, MagicErr> {
    let mut magics = Vec::<Magic>::with_capacity(64);
    for s in 0..64 {
        let magic = find_magic(s, mask_fn, attacks_fn, magic_iter)?;
        magics.push(magic);
    }
    Ok(MagicAttacks { magics })
}

/// Finds a Magic for a given square.
///
/// # Arguments
///
/// * `mask_fn`: A function to compute the full mask for a piece given a square.
/// * `attacks_fn`: A function to compute the attack mask for a piece given a square and a given
/// set of blockers.
/// * `magic_iter`: An iterator over magic numbers.
fn find_magic(
    sq: u32,
    mask_fn: &impl Fn(u32) -> BitBoard,
    attacks_fn: &impl Fn(u32, BitBoard) -> BitBoard,
    magic_iter: &mut impl Iterator<Item = u64>,
) -> Result<Magic, MagicErr> {
    if sq >= 64 {
        return Err(MagicErr::InvalidSquare);
    }

    let mask = mask_fn(sq);
    let num_bits = mask.count();

    if num_bits < 5 || num_bits > 12 {
        return Err(MagicErr::NumBits);
    }

    let ncombos = 1usize << num_bits;
    const MAX_COMBOS: usize = 1usize << 12;

    let mut blocking = [BitBoard::new(); MAX_COMBOS];
    let mut attacking = [BitBoard::new(); MAX_COMBOS];

    for i in 0..ncombos {
        blocking[i] = permute_mask(BitBoard::from(i as u64), mask);
        attacking[i] = attacks_fn(sq, blocking[i]);
    }

    let mut attacks = vec![BitBoard::new(); ncombos];

    'mloop: for magic in magic_iter {
        if ((mask * magic) >> 56).count() < 6 {
            continue;
        }

        let rshift = 64 - num_bits;

        for i in 0..ncombos {
            let magic_hash = get_magic_hash(blocking[i], magic, rshift);

            if attacks[magic_hash].none() {
                attacks[magic_hash] = attacking[i];
            } else if attacks[magic_hash] != attacking[i] {
                // Start again if a collision is found.
                for b in &mut attacks {
                    b.clear();
                }
                continue 'mloop;
            }
        }

        return Ok(Magic {
            attacks,
            mask,
            magic,
            rshift,
        });
    }

    Err(MagicErr::NotFound)
}

#[inline(always)]
fn get_magic_hash(blocking: BitBoard, magic: u64, rshift: u32) -> usize {
    ((blocking * magic) >> rshift).u64() as usize
}

pub fn create_rand_iter() -> impl Iterator<Item = u64> {
    let mut rng = rand::thread_rng();
    std::iter::from_fn(move || Some(rng.next_u64() & rng.next_u64() & rng.next_u64()))
}

/// The bits in bit_selector are used to choose a set of bits from mask.
pub fn permute_mask(bit_selector: BitBoard, mask: BitBoard) -> BitBoard {
    let mut bits = BitBoard::new();
    for (i, index) in mask.sq_iter().enumerate() {
        if bit_selector.has_bit(i as u32) {
            bits.set_bit(index);
        }
    }
    bits
}

/// Computes a BitBoard with the full bishop mask for a given square, but ignores the outer edge
/// squares and the current square. For example, given square 0, i.e. square a1, it returns a
/// BitBoard with bits set at (b2, c3, d4, e5, f6, g7).
pub fn get_full_bmask(square: u32) -> BitBoard {
    let row = square / 8;
    let col = square % 8;
    let mut bits = BitBoard::new();
    // up-right
    for (r, c) in (row + 1..7).zip(col + 1..7) {
        bits.set_bit(from_rc(r, c));
    }
    // down-right
    for (r, c) in (1..row).rev().zip(col + 1..7) {
        bits.set_bit(from_rc(r, c));
    }
    // down-left
    for (r, c) in (1..row).rev().zip((1..col).rev()) {
        bits.set_bit(from_rc(r, c));
    }
    // up-left
    for (r, c) in (row + 1..7).zip((1..col).rev()) {
        bits.set_bit(from_rc(r, c));
    }
    bits
}

/// Computes a vector of 64 BitBoards, each representing the full bishop mask for a square.
pub fn get_full_bmasks() -> Vec<BitBoard> {
    (0..64).map(|i| get_full_bmask(i)).collect()
}

/// Computes the set of squares that are attacked by a bishop from a given square given a set of
/// blocking pieces.
pub fn get_battacks(sq: u32, blocking: BitBoard) -> BitBoard {
    let row = sq / 8;
    let col = sq % 8;
    let mut attacks = BitBoard::new();
    // up-right
    for i in (row + 1..=7).zip(col + 1..=7).map(|(r, c)| from_rc(r, c)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // down-right
    for i in (0..row).rev().zip(col + 1..=7).map(|(r, c)| from_rc(r, c)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // down-left
    for i in (0..row)
        .rev()
        .zip((0..col).rev())
        .map(|(r, c)| from_rc(r, c))
    {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // up-left
    for i in (row + 1..=7)
        .zip((0..col).rev())
        .map(|(r, c)| from_rc(r, c))
    {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    attacks
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
        bits.set_bit(from_rc(r, col));
    }
    // right
    for c in col + 1..7 {
        bits.set_bit(from_rc(row, c));
    }
    // down
    for r in 1..row {
        bits.set_bit(from_rc(r, col));
    }
    // left
    for c in 1..col {
        bits.set_bit(from_rc(row, c));
    }
    bits
}

/// Computes a vector of 64 BitBoards, each representing the full rook mask for a square.
pub fn get_full_rmasks() -> Vec<BitBoard> {
    (0..64).map(|i| get_full_rmask(i)).collect()
}

/// Computes the set of squares that are attacked by a rook from a given square given a set of
/// blocking pieces.
pub fn get_rattacks(sq: u32, blocking: BitBoard) -> BitBoard {
    let row = sq / 8;
    let col = sq % 8;
    let mut attacks = BitBoard::new();
    // up
    for i in (row + 1..=7).map(|r| from_rc(r, col)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // right
    for i in (col + 1..=7).map(|c| from_rc(row, c)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // down
    for i in (0..row).rev().map(|r| from_rc(r, col)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    // left
    for i in (0..col).rev().map(|c| from_rc(row, c)) {
        attacks.set_bit(i);
        if blocking.has_bit(i) {
            break;
        }
    }
    attacks
}

#[inline(always)]
fn from_rc(row: u32, col: u32) -> u32 {
    row * 8 + col
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
    fn get_bishop_attacks_from_e4() {
        let blockers = BitBoard::from(&[D3, D5, F3, F5]);
        let attacks = get_battacks(E4, blockers);
        assert_eq!(attacks, blockers);
    }

    #[test]
    fn get_bishop_attacks_from_d4() {
        let blockers = BitBoard::from(&[G7, F2]);
        let attacks = get_battacks(D4, blockers);
        assert_eq!(
            attacks,
            BitBoard::from(&[A1, B2, C3, E3, F2, C5, E5, B6, F6, A7, G7])
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

    #[test]
    fn get_rook_attacks_from_e4() {
        let blockers = BitBoard::from(&[D4, E5, F4, E3]);
        let attacks = get_rattacks(E4, blockers);
        assert_eq!(attacks, blockers);
    }

    #[test]
    fn get_rook_attacks_from_d4() {
        let blockers = BitBoard::from(&[A4, B4, F4, D3]);
        let attacks = get_rattacks(D4, blockers);
        assert_eq!(
            attacks,
            BitBoard::from(&[D3, B4, C4, E4, F4, D5, D6, D7, D8])
        );
    }

    #[test]
    fn create_rand_iterator() {
        let iter = create_rand_iter();
        for m in iter.take(3) {
            assert!(m > 0);
        }
    }

    #[test]
    fn permuate_mask() {
        let bit_selector = BitBoard::from(0b100);
        let mask = BitBoard::from(0b111000);
        assert_eq!(permute_mask(bit_selector, mask), BitBoard::from(0b100000));
    }
}

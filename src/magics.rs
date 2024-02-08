use crate::bitboard::BitBoard;
use crate::sq::Sq;
use rand::RngCore;
use std::ops::Fn;

#[derive(Copy, Clone, Debug)]
pub enum MagicErr {
    InvalidSquare,
    NumBits,
    NumMagic,
    NotFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Magic {
    pub attacks: Vec<BitBoard>,
    pub mask: BitBoard,
    pub magic: u64,
    pub rshift: u8,
}

impl Magic {
    fn get_hash(&self, blockers: BitBoard) -> usize {
        let b = blockers & self.mask;
        get_magic_hash(b, self.magic, self.rshift)
    }

    fn get_attacks(&self, blockers: BitBoard) -> Option<BitBoard> {
        let h = self.get_hash(blockers);
        self.attacks.get(h).copied()
    }
}

pub trait Magics: AsRef<[Magic]> {
    fn attacks(&self, sq: Sq, blockers: BitBoard) -> Option<BitBoard>;
    fn get(&self, sq: Sq) -> Option<&Magic>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MagicAttacks {
    magics: Vec<Magic>,
}

impl Magics for MagicAttacks {
    fn attacks(&self, sq: Sq, blockers: BitBoard) -> Option<BitBoard> {
        let magic = self.magics.get(usize::from(sq))?;
        magic.get_attacks(blockers)
    }

    fn get(&self, sq: Sq) -> Option<&Magic> {
        self.magics.get(usize::from(sq))
    }
}

impl AsRef<[Magic]> for MagicAttacks {
    fn as_ref(&self) -> &[Magic] {
        self.magics.as_ref()
    }
}

pub fn from_bmagics(mut magics: impl Iterator<Item = u64>) -> Result<MagicAttacks, MagicErr> {
    if magics.size_hint().0 != 64 {
        return Err(MagicErr::NumMagic);
    }
    find_all_magics(&get_full_bmask, &get_battacks, magics.by_ref())
}

pub fn from_rmagics(mut magics: impl Iterator<Item = u64>) -> Result<MagicAttacks, MagicErr> {
    if magics.size_hint().0 != 64 {
        return Err(MagicErr::NumMagic);
    }
    find_all_magics(&get_full_rmask, &get_rattacks, magics.by_ref())
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
    mask_fn: &impl Fn(Sq) -> BitBoard,
    attacks_fn: &impl Fn(Sq, BitBoard) -> BitBoard,
    magic_iter: &mut impl Iterator<Item = u64>,
) -> Result<MagicAttacks, MagicErr> {
    let mut magics = Vec::<Magic>::with_capacity(64);
    for s in 0u8..64 {
        let magic = find_magic(s.into(), mask_fn, attacks_fn, magic_iter)?;
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
    sq: Sq,
    mask_fn: &impl Fn(Sq) -> BitBoard,
    attacks_fn: &impl Fn(Sq, BitBoard) -> BitBoard,
    magic_iter: &mut impl Iterator<Item = u64>,
) -> Result<Magic, MagicErr> {
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

        let rshift: u8 = 64 - num_bits;

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
fn get_magic_hash(blocking: BitBoard, magic: u64, rshift: u8) -> usize {
    ((blocking * magic) >> rshift).u64() as usize
}

pub fn create_rand_iter() -> impl Iterator<Item = u64> {
    let mut rng = rand::thread_rng();
    std::iter::from_fn(move || Some(rng.next_u64() & rng.next_u64() & rng.next_u64()))
}

/// The bits in bit_selector are used to choose a set of bits from mask.
pub fn permute_mask(bit_selector: BitBoard, mask: BitBoard) -> BitBoard {
    let mut bits = BitBoard::new();
    for (i, sq) in mask.sq_iter().enumerate() {
        if bit_selector.has_bit(i.into()) {
            bits |= sq;
        }
    }
    bits
}

/// Computes a BitBoard with the full bishop mask for a given square, but ignores the outer edge
/// squares and the current square. For example, given square 0, i.e. square a1, it returns a
/// BitBoard with bits set at (b2, c3, d4, e5, f6, g7).
pub fn get_full_bmask(sq: Sq) -> BitBoard {
    let (row, col) = sq.rc();
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
    (0u8..64).map(|i| get_full_bmask(i.into())).collect()
}

/// Computes the set of squares that are attacked by a bishop from a given square given a set of
/// blocking pieces.
pub fn get_battacks(sq: Sq, blocking: BitBoard) -> BitBoard {
    let (row, col) = sq.rc();
    let mut attacks = BitBoard::new();
    // up-right
    for s in (row + 1..=7).zip(col + 1..=7).map(|(r, c)| from_rc(r, c)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // down-right
    for s in (0..row).rev().zip(col + 1..=7).map(|(r, c)| from_rc(r, c)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // down-left
    for s in (0..row)
        .rev()
        .zip((0..col).rev())
        .map(|(r, c)| from_rc(r, c))
    {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // up-left
    for s in (row + 1..=7)
        .zip((0..col).rev())
        .map(|(r, c)| from_rc(r, c))
    {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    attacks
}

/// Computes a BitBoard with the full rook mask for a given square, but ignores the outer edge
/// squares and the current square. For example, given square 0, i.e. square a1, it returns a
/// BitBoard with bits set at (a2, a3, a4, a5, a6, a7, b1, c1, d1, e1, f1, g1).
pub fn get_full_rmask(sq: Sq) -> BitBoard {
    let (row, col) = sq.rc();
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
    (0u8..64).map(|i| get_full_rmask(i.into())).collect()
}

/// Computes the set of squares that are attacked by a rook from a given square given a set of
/// blocking pieces.
pub fn get_rattacks(sq: Sq, blocking: BitBoard) -> BitBoard {
    let (row, col) = sq.rc();
    let mut attacks = BitBoard::new();
    // up
    for s in (row + 1..=7).map(|r| from_rc(r, col)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // right
    for s in (col + 1..=7).map(|c| from_rc(row, c)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // down
    for s in (0..row).rev().map(|r| from_rc(r, col)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    // left
    for s in (0..col).rev().map(|c| from_rc(row, c)) {
        attacks.set_bit(s);
        if blocking.has_bit(s) {
            break;
        }
    }
    attacks
}

// Generated by gen_matics.rs.
const BMAGICS: [u64; 64] = [
    0x840828802082140,
    0x20220c83010004,
    0x4140c00451041,
    0x5248204040000112,
    0x804050489200000,
    0x4102080308108002,
    0x8104022202200108,
    0x13004050480804,
    0x300080810140855,
    0xa0001041c2088200,
    0x2080899060404,
    0x1110408806210,
    0x10221210008000,
    0x8221200280,
    0x8042016114202080,
    0x420222010b29104c,
    0x22404002041100,
    0xa000040800a100,
    0x802041008320020,
    0x2800810808210004,
    0x1206001012100800,
    0x8100400808080420,
    0x521201042000,
    0x4000202101011000,
    0x10048050149001,
    0x118420c28109103,
    0xa202445108080100,
    0x2080004004008,
    0xc210802002020043,
    0x4001010022008080,
    0xc28108021008800,
    0x22002490808801,
    0x4200602081000,
    0x4108200880222,
    0x126002400020800,
    0xa0400808008200,
    0x4028020400004500,
    0x8202082a00104047,
    0x8109100040140,
    0x4041004491020200,
    0x22080a2210102081,
    0x2084208040200,
    0xd1004050010844,
    0x10141041800,
    0x10a0040810100202,
    0x1102600800801108,
    0x8c20a24c03040050,
    0xc02440c10810820,
    0x4080880442a00084,
    0x10b0840098840a82,
    0x40104a0203311500,
    0x8000460884240043,
    0x1000110803040024,
    0x2800040408020400,
    0x4048404040024,
    0x20140120490000,
    0x104210104800,
    0x28113208022800,
    0x3200000100880418,
    0x2041218008840444,
    0x60c80910020210,
    0x3208800545080200,
    0x2001a02004910040,
    0x40015444820240,
];

// Generated by gen_matics.rs.
const RMAGICS: [u64; 64] = [
    0x1080001080400024,
    0xa100208040001100,
    0x200100a22014080,
    0x100100100082004,
    0x200200200040910,
    0x500040008020100,
    0x80020000800100,
    0x4080010000403080,
    0x802040008006,
    0x4802008400080,
    0x9000801000802000,
    0x9801002100081000,
    0x4808008000400,
    0x2010808002000400,
    0x4002800200110080,
    0x1000040820100,
    0x1c22880008cc000,
    0xa010004000200048,
    0x2850010100402000,
    0x3480848008009000,
    0x114008004080080,
    0x4008080020004,
    0x14040041023810,
    0x810020004008041,
    0x2400802080004000,
    0x8001020200408021,
    0x1a00860200144020,
    0x1010008080080010,
    0x6000080100110004,
    0x42020080800400,
    0x2010010080800200,
    0x10200104084,
    0xa40400081800034,
    0x30804000802000,
    0x200441001501,
    0x10001080800804,
    0x1400040080800800,
    0x8006001812003044,
    0x4180114002210,
    0x41042000081,
    0x400080008020,
    0x2000205004c000,
    0x4800200900410011,
    0x800a0040210a0012,
    0x8004040008008080,
    0x400a000410020008,
    0x40200010100,
    0x2000050840820024,
    0x800840002480,
    0x208240030300,
    0x802004100880,
    0x220080180100180,
    0x200800400080080,
    0x120800200040080,
    0x90080102100400,
    0xb23000082004100,
    0x70800024104501,
    0x6124300220082,
    0x2000ac010a08202,
    0x403000420081001,
    0x43004800341251,
    0x12001048410402,
    0x9200192088421004,
    0x80100440981a2,
];

#[inline(always)]
fn from_rc(row: u8, col: u8) -> Sq {
    Sq::from(row * 8 + col)
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
            masks[usize::from(A1)],
            BitBoard::from(&[B2, C3, D4, E5, F6, G7])
        );
        assert_eq!(
            masks[usize::from(E5)],
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
            masks[usize::from(A1)],
            BitBoard::from(&[A2, A3, A4, A5, A6, A7, B1, C1, D1, E1, F1, G1])
        );
        assert_eq!(
            masks[usize::from(E5)],
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

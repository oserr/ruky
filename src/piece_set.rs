use crate::bitboard::BitBoard;
use crate::piece::{Piece, Piece::*};
use crate::sq;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PieceSet {
    king: BitBoard,
    queen: BitBoard,
    rook: BitBoard,
    bishop: BitBoard,
    knight: BitBoard,
    pawn: BitBoard,
    all_bits: BitBoard,
}

impl PieceSet {
    pub fn init_white() -> Self {
        Self {
            king: BitBoard::from(1 << sq::E1),
            queen: BitBoard::from(1 << sq::D1),
            rook: BitBoard::from((1 << sq::H1) | (1 << sq::A1)),
            bishop: BitBoard::from((1 << sq::F1) | (1 << sq::C1)),
            knight: BitBoard::from((1 << sq::G1) | (1 << sq::B1)),
            pawn: BitBoard::from(0xff00),
            all_bits: BitBoard::from(0xffff),
        }
    }

    pub fn init_black() -> Self {
        Self {
            king: BitBoard::from(1 << sq::E8),
            queen: BitBoard::from(1 << sq::D8),
            rook: BitBoard::from((1 << sq::H8) | (1 << sq::A8)),
            bishop: BitBoard::from((1 << sq::F8) | (1 << sq::C8)),
            knight: BitBoard::from((1 << sq::G8) | (1 << sq::B8)),
            pawn: BitBoard::from(0xff << 48),
            all_bits: BitBoard::from(0xffff << 48),
        }
    }

    pub fn iter(&self) -> PieceIter {
        PieceIter::from(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PieceIter<'a> {
    pieces: &'a PieceSet,
    current: Option<Piece<BitBoard>>,
}

impl<'a> From<&'a PieceSet> for PieceIter<'a> {
    fn from(pieces: &'a PieceSet) -> PieceIter<'a> {
        PieceIter {
            pieces: pieces,
            current: Some(King(pieces.king)),
        }
    }
}

impl<'a> Iterator for PieceIter<'a> {
    type Item = Piece<BitBoard>;

    fn next(&mut self) -> Option<Piece<BitBoard>> {
        match self.current {
            None => None,
            Some(piece) => {
                self.current = match piece {
                    King(_) => Some(Queen(self.pieces.queen)),
                    Queen(_) => Some(Rook(self.pieces.rook)),
                    Rook(_) => Some(Bishop(self.pieces.bishop)),
                    Bishop(_) => Some(Knight(self.pieces.knight)),
                    Knight(_) => Some(Pawn(self.pieces.pawn)),
                    Pawn(_) => None,
                };
                Some(piece)
            }
        }
    }
}

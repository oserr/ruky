use crate::bitboard::BitBoard;
use crate::piece::{Piece, Piece::*};

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

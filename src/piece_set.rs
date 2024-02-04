use crate::bitboard::BitBoard;
use crate::piece::{Piece, Piece::*};
use crate::piece_move::{MoveErr, PieceMove, PieceMove::*};
use crate::sq;

/// PieceSet represents the set of pieces for player, with a bitboard for each type of piece.
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
    // Initializes a PieceSet with the initial position for white pieces.
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

    // Initializes a PieceSet with the initial position for black pieces.
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

    // Updates the position of a piece after a move is made. This is only for the side making the
    // move, so captures need to be handled by the PieceSet of the other pieces. Returns an error
    // if the move is not valid, e.g. the piece being moved is not found on the source square.
    pub fn apply_move(&mut self, piece_move: Piece<PieceMove>) -> Result<&mut Self, MoveErr> {
        match piece_move {
            King(mv) => self.update_king(mv),
            Queen(mv) => self.simple_update(mv, Queen(())),
            Rook(mv) => self.simple_update(mv, Rook(())),
            Bishop(mv) => self.simple_update(mv, Bishop(())),
            Knight(mv) => self.simple_update(mv, Knight(())),
            Pawn(mv) => self.update_pawn(mv),
        }
    }

    // Updates the position for the king. Note that this also handles castling. Returns an error if
    // the move is not valid.
    fn update_king(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Simple { from, to } | Capture { from, to, .. } => {
                self.king.update_bit(from.into(), to.into())?
            }
            Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            } => {
                self.king.update_bit(king_from.into(), king_to.into())?;
                self.rook.update_bit(rook_from.into(), rook_to.into())?
            }
            _ => return Err(MoveErr::BadMove(King(mv))),
        };
        Ok(self)
    }

    // Updates the position a pawn. Returns an error if the move is not valid.
    fn update_pawn(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Simple { from, to } | Capture { from, to, .. } | EnPassant { from, to, .. } => {
                self.pawn.update_bit(from.into(), to.into())?
            }
            Promo { from, to, promo }
            | PromoCap {
                from, to, promo, ..
            } => {
                self.pawn.clear_bit_or(from.into())?;
                let promo_piece = match promo {
                    Queen(_) => &mut self.queen,
                    Rook(_) => &mut self.rook,
                    Bishop(_) => &mut self.bishop,
                    Knight(_) => &mut self.knight,
                    _ => return Err(MoveErr::BadPromo(promo)),
                };
                promo_piece.set_bit_or(to.into())?
            }
            _ => return Err(MoveErr::BadMove(Pawn(mv))),
        };
        Ok(self)
    }

    // Updates the position for pieces with simple moves: queens, rooks, bishops, and knights.
    // Returns an error for invalid moves.
    fn simple_update(
        &mut self,
        mv: PieceMove,
        piece_type: Piece<()>,
    ) -> Result<&mut Self, MoveErr> {
        let piece = match piece_type {
            Queen(_) => &mut self.queen,
            Rook(_) => &mut self.rook,
            Bishop(_) => &mut self.bishop,
            Knight(_) => &mut self.knight,
            _ => panic!("Using simple update for {:?}", piece_type),
        };
        match mv {
            Simple { from, to } | Capture { from, to, .. } => {
                piece.update_bit(from.into(), to.into())?
            }
            _ => {
                let piece_move = match piece_type {
                    Queen(_) => Queen(mv),
                    Rook(_) => Rook(mv),
                    Bishop(_) => Bishop(mv),
                    Knight(_) => Knight(mv),
                    _ => panic!("Using simple update for {:?}", piece_type),
                };
                return Err(MoveErr::BadMove(piece_move));
            }
        };
        Ok(self)
    }

    // Updates the position for pieces that are captured. If the move is not a capture or the
    // capture is invalid, then it returns an error.
    pub fn remove_captured(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Capture { to, cap, .. } | PromoCap { to, cap, .. } => {
                let piece = match cap {
                    King(_) => &mut self.king,
                    Queen(_) => &mut self.queen,
                    Rook(_) => &mut self.rook,
                    Bishop(_) => &mut self.bishop,
                    Knight(_) => &mut self.knight,
                    Pawn(_) => &mut self.pawn,
                };
                piece.clear_bit_or(to.into())?
            }
            EnPassant { passant, .. } => self.pawn.clear_bit_or(passant.into())?,
            _ => return Err(MoveErr::NoCapture(mv)),
        };
        Ok(self)
    }

    // Returns an iterator to iterate over each piece as a BitBoard.
    pub fn iter(&self) -> PieceIter {
        PieceIter::from(self)
    }
}

// A helper struct to make it easy to iterate over a PieceSet.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PieceIter<'a> {
    pieces: &'a PieceSet,
    current: Option<Piece<BitBoard>>,
}

// Converts a PieceSet into a PieceIter.
impl<'a> From<&'a PieceSet> for PieceIter<'a> {
    // The iterator always starts with the King.
    fn from(pieces: &'a PieceSet) -> PieceIter<'a> {
        PieceIter {
            pieces: pieces,
            current: Some(King(pieces.king)),
        }
    }
}

// Implement iteration for PieceIter.
impl<'a> Iterator for PieceIter<'a> {
    type Item = Piece<BitBoard>;

    // Pieces are traversed in the same order of the fields in the struct.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_white_pieces() {
        let pieces = PieceSet::init_white();
        for piece in pieces.iter() {
            match piece {
                King(king) => {
                    assert_eq!(king.count(), 1);
                    assert_eq!(king.take_first(), Some(sq::E1))
                }
                Queen(queen) => {
                    assert_eq!(queen.count(), 1);
                    assert_eq!(queen.take_first(), Some(sq::D1))
                }
                Rook(rook) => assert_eq!(rook.to_vec::<u8>(), vec![sq::A1, sq::H1]),
                Bishop(bishop) => assert_eq!(bishop.to_vec::<u8>(), vec![sq::C1, sq::F1]),
                Knight(knight) => assert_eq!(knight.to_vec::<u8>(), vec![sq::B1, sq::G1]),
                Pawn(pawn) => assert_eq!(pawn.to_vec::<u8>(), (8..=15).collect()),
            };
        }
    }
}

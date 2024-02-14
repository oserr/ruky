use crate::bitboard::BitBoard;
use crate::magics::ChessMagics;
use crate::piece::{Color, Piece, Piece::*};
use crate::piece_move::{MoveErr, PieceMove, PieceMove::*};
use crate::sq::{self, Sq};

/// PieceSet represents the set of pieces for player, with a bitboard for each
/// type of piece.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PieceSet {
    king: BitBoard,
    queen: BitBoard,
    rook: BitBoard,
    bishop: BitBoard,
    knight: BitBoard,
    pawn: BitBoard,
    all_bits: BitBoard,
    color: Color,
    king_castle: bool,
    queen_castle: bool,
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
            color: Color::White,
            king_castle: true,
            queen_castle: true,
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
            color: Color::Black,
            king_castle: true,
            queen_castle: true,
        }
    }

    pub fn king(&self) -> BitBoard {
        self.king
    }

    pub fn knight(&self) -> BitBoard {
        self.knight
    }

    pub fn all(&self) -> BitBoard {
        self.all_bits
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn find_type(&self, sq: Sq) -> Option<Piece<()>> {
        self.iter().find_map(|pt| Some(pt.with(())))
    }

    pub fn attacks(&self, other: &PieceSet, magics: &ChessMagics) -> AttackSquares {
        assert_ne!(self.color, other.color);

        let all_blockers = self.all_bits | other.all_bits;
        let empty = !all_blockers;

        let mut pieces = BitBoard::new();
        let mut no_pieces = BitBoard::new();

        // King attack squares.
        let moves = self.king.king_moves();
        pieces |= moves & other.all_bits;
        no_pieces |= moves & empty;

        // Knight attack squares.
        let moves = self.knight.knight_moves();
        pieces |= moves & other.all_bits;
        no_pieces |= moves & empty;

        if self.color == Color::White {
            // White pawn attack squares.
            let moves = self.pawn.wp_left();
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;

            let moves = self.pawn.wp_right();
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;
        } else {
            // Black pawn attack squares.
            let moves = self.pawn.bp_left();
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;

            let moves = self.pawn.bp_right();
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;
        }

        // Bishop attack squares.
        for sq in self.bishop.sq_iter() {
            let moves = magics
                .bmagics(sq, all_blockers)
                .expect("Unable to compute bishop magics");
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;
        }

        // Rook attack squares.
        for sq in self.rook.sq_iter() {
            let moves = magics
                .rmagics(sq, all_blockers)
                .expect("Unable to compute rook magics");
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;
        }

        // Queen attack squares.
        for sq in self.queen.sq_iter() {
            let moves = magics
                .qmagics(sq, all_blockers)
                .expect("Unable to compute queen magics");
            pieces |= moves & other.all_bits;
            no_pieces |= moves & empty;
        }

        AttackSquares { pieces, no_pieces }
    }

    // Updates the position of a piece after a move is made. This is only for the
    // side making the move, so captures need to be handled by the PieceSet of
    // the other pieces. Returns an error if the move is not valid, e.g. the
    // piece being moved is not found on the source square.
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

    // Updates the position for the king. Note that this also handles castling.
    // Returns an error if the move is not valid.
    fn update_king(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Simple { from, to } | Capture { from, to, .. } => {
                self.king.update_bit(from, to)?;
                self.all_bits.update_bit(from, to)?
            }
            Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            } => {
                self.king.update_bit(king_from, king_to)?;
                self.rook.update_bit(rook_from, rook_to)?;
                self.all_bits.update_bit(king_from, king_to)?;
                self.all_bits.update_bit(rook_from, rook_to)?
            }
            _ => return Err(MoveErr::BadMove(King(mv))),
        };
        self.king_castle = false;
        self.queen_castle = false;
        Ok(self)
    }

    // Updates the position a pawn. Returns an error if the move is not valid.
    fn update_pawn(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Simple { from, to } | Capture { from, to, .. } | EnPassant { from, to, .. } => {
                self.pawn.update_bit(from, to)?;
                self.all_bits.update_bit(from, to)?
            }
            Promo { from, to, promo }
            | PromoCap {
                from, to, promo, ..
            } => {
                self.pawn.clear_bit_or(from)?;
                let promo_piece = match promo {
                    Queen(_) => &mut self.queen,
                    Rook(_) => &mut self.rook,
                    Bishop(_) => &mut self.bishop,
                    Knight(_) => &mut self.knight,
                    _ => return Err(MoveErr::BadPromo(promo)),
                };
                promo_piece.set_bit_or(to)?;
                self.all_bits.update_bit(from, to)?
            }
            _ => return Err(MoveErr::BadMove(Pawn(mv))),
        };
        Ok(self)
    }

    // Updates the position for pieces with simple moves: queens, rooks, bishops,
    // and knights. Returns an error for invalid moves.
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
                // TODO: Can this check be removed every time we make a simple update?
                if piece_type.is_rook() {
                    // Maybe remove castling right for a specific rook.
                    match (self.color, from) {
                        (Color::White, sq::A1) | (Color::Black, sq::A8) => {
                            self.queen_castle = false
                        }
                        (Color::White, sq::H1) | (Color::Black, sq::H8) => self.king_castle = false,
                        _ => (),
                    };
                }
                piece.update_bit(from, to)?;
                self.all_bits.update_bit(from, to)?
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

    // Updates the position for pieces that are captured. If the move is not a
    // capture or the capture is invalid, then it returns an error.
    pub fn remove_captured(&mut self, mv: PieceMove) -> Result<&mut Self, MoveErr> {
        match mv {
            Capture { to, cap, .. } | PromoCap { to, cap, .. } => {
                let piece = match cap {
                    King(_) => &mut self.king,
                    Queen(_) => &mut self.queen,
                    Rook(_) => {
                        // If we are removing a rook that has not moved, then we need to remove the
                        // the castling right for that side.
                        match (self.color, to) {
                            (Color::White, sq::A1) | (Color::Black, sq::A8) => {
                                self.queen_castle = false;
                            }
                            (Color::White, sq::H1) | (Color::Black, sq::H8) => {
                                self.king_castle = false;
                            }
                            _ => (),
                        };
                        &mut self.rook
                    }
                    Bishop(_) => &mut self.bishop,
                    Knight(_) => &mut self.knight,
                    Pawn(_) => &mut self.pawn,
                };
                piece.clear_bit_or(to)?;
                self.all_bits.clear_bit_or(to)?
            }
            EnPassant { passant, .. } => {
                self.pawn.clear_bit_or(passant)?;
                self.all_bits.clear_bit_or(passant)?
            }
            _ => return Err(MoveErr::NoCapture(mv)),
        };
        Ok(self)
    }

    // Returns a pair of optional moves for king and queen side castling if they are
    // valid, which means that the king or rook have not lost the right to
    // castle, there are no pieces between the king and rook, and the squares
    // between the king and rook are not being attacked.
    //
    // @param other The opposing pieces.
    // @param attacked A BitBoard representing all the squares that are attacked by
    // the other pieces. @return A pair in the form of (king side castle, queen
    // side castle). The moves are only set if they are valid.
    pub fn castle(
        &self,
        other: &PieceSet,
        attacked: BitBoard,
    ) -> (Option<Piece<PieceMove>>, Option<Piece<PieceMove>>) {
        assert_ne!(self.color, other.color);

        if !self.king_castle && !self.queen_castle {
            return (None, None);
        }

        let mut blocked = self.all() | other.all() | attacked;
        if self.color == Color::Black {
            blocked >>= 56;
        }

        (
            self.try_king_castle(blocked),
            self.try_queen_castle(blocked),
        )
    }

    // Computes the move for king side castling, if valid, otherwise returns None.
    //
    // @param blocked A bitboard represeting all the blocked squares because they
    // are either occupied or attacked by the other pieces. @return The king
    // castling move if valid, or None.
    fn try_king_castle(&self, blocked: BitBoard) -> Option<Piece<PieceMove>> {
        if !self.king_castle {
            return None;
        }

        // The bit pattern representing no pieces between the king and king side rook.
        let king_bits = BitBoard::from(0b10010000u64);

        // The bit pattern to mask all squares between the king and king side rook.
        let king_mask = BitBoard::from(0b11110000u64);

        if (blocked & king_mask) != king_bits {
            None
        } else {
            let (king_from, king_to, rook_from, rook_to) = if self.color == Color::White {
                (sq::E1, sq::G1, sq::H1, sq::F1)
            } else {
                (sq::E8, sq::G8, sq::H8, sq::F8)
            };
            Some(King(Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            }))
        }
    }

    // Computes the move for queen side castling, if valid, otherwise returns None.
    //
    // @param blocked A bitboard represeting all the blocked squares because they
    // are either occupied or attacked by the other pieces. @return The king
    // castling move if valid, or None.
    fn try_queen_castle(&self, blocked: BitBoard) -> Option<Piece<PieceMove>> {
        if !self.queen_castle {
            return None;
        }

        // The bit pattern representing no pieces between the king and queen side rook.
        let queen_bits = BitBoard::from(0b00010001u64);

        // The bit pattern to mask all squares between the king and queen side rook.
        let queen_mask = BitBoard::from(0b00011111u64);

        if (blocked & queen_mask) != queen_bits {
            None
        } else {
            let (king_from, king_to, rook_from, rook_to) = if self.color == Color::White {
                (sq::E1, sq::C1, sq::A1, sq::D1)
            } else {
                (sq::E8, sq::C8, sq::A8, sq::D8)
            };
            Some(King(Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            }))
        }
    }

    // Returns an iterator to iterate over each piece as a BitBoard.
    pub fn iter(&self) -> PieceIter {
        PieceIter::from(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AttackSquares {
    pub pieces: BitBoard,
    pub no_pieces: BitBoard,
}

// A helper struct to make it easy to iterate over a PieceSet.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PieceIter<'a> {
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
    use crate::sq::Sq;

    #[test]
    fn init_white_pieces() {
        let pieces = PieceSet::init_white();
        for piece in pieces.iter() {
            match piece {
                King(mut king) => {
                    assert_eq!(king.count(), 1);
                    assert_eq!(king.take_first(), Some(sq::E1))
                }
                Queen(mut queen) => {
                    assert_eq!(queen.count(), 1);
                    assert_eq!(queen.take_first(), Some(sq::D1))
                }
                Rook(rook) => assert_eq!(Vec::<Sq>::from(rook), vec![sq::A1, sq::H1]),
                Bishop(bishop) => assert_eq!(Vec::<Sq>::from(bishop), vec![sq::C1, sq::F1]),
                Knight(knight) => assert_eq!(Vec::<Sq>::from(knight), vec![sq::B1, sq::G1]),
                Pawn(pawn) => {
                    assert_eq!(
                        Vec::<Sq>::from(pawn),
                        (8u8..=15).map(Sq::from).collect::<Vec<_>>()
                    )
                }
            };
        }
        assert_eq!(
            Vec::<Sq>::from(pieces.all()),
            (0u8..16).map(Sq::from).collect::<Vec<_>>()
        );
        assert_eq!(pieces.color, Color::White);
    }

    #[test]
    fn init_black_pieces() {
        let pieces = PieceSet::init_black();
        for piece in pieces.iter() {
            match piece {
                King(mut king) => {
                    assert_eq!(king.count(), 1);
                    assert_eq!(king.take_first(), Some(sq::E8))
                }
                Queen(mut queen) => {
                    assert_eq!(queen.count(), 1);
                    assert_eq!(queen.take_first(), Some(sq::D8))
                }
                Rook(rook) => assert_eq!(Vec::<Sq>::from(rook), vec![sq::A8, sq::H8]),
                Bishop(bishop) => assert_eq!(Vec::<Sq>::from(bishop), vec![sq::C8, sq::F8]),
                Knight(knight) => assert_eq!(Vec::<Sq>::from(knight), vec![sq::B8, sq::G8]),
                Pawn(pawn) => {
                    assert_eq!(
                        Vec::<Sq>::from(pawn),
                        (48u8..=55).map(Sq::from).collect::<Vec<_>>()
                    )
                }
            };
        }
        assert_eq!(
            Vec::<Sq>::from(pieces.all()),
            (48u8..64).map(Sq::from).collect::<Vec<_>>()
        );
        assert_eq!(pieces.color, Color::Black);
    }

    #[test]
    #[ignore]
    fn init_attacks() {
        let white = PieceSet::init_white();
        let black = PieceSet::init_black();
        let magics = ChessMagics::from_precomputed().expect("Unable to compute magics.");

        let white_attacks = white.attacks(&black, &magics);

        assert_eq!(white_attacks.pieces, BitBoard::new());
        assert_eq!(
            white_attacks.no_pieces,
            BitBoard::from(&[
                sq::A3,
                sq::B3,
                sq::C3,
                sq::D3,
                sq::E3,
                sq::F3,
                sq::G3,
                sq::H3
            ])
        );

        let black_attacks = black.attacks(&white, &magics);

        assert_eq!(black_attacks.pieces, BitBoard::new());
        assert_eq!(
            black_attacks.no_pieces,
            BitBoard::from(&[
                sq::A6,
                sq::B6,
                sq::C6,
                sq::D6,
                sq::E6,
                sq::F6,
                sq::G6,
                sq::H6
            ])
        );
    }
}

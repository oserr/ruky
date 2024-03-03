use crate::board::{Board, BoardBuilder};
use crate::fen::{from_fen, FenErr};
use crate::magics::ChessMagics;
use crate::piece::Piece;
use crate::piece_move::PieceMove;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Ruky {
    magics: Arc<ChessMagics>,
}

impl Ruky {
    pub fn new() -> Self {
        Self {
            magics: Arc::new(
                ChessMagics::from_precomputed().expect("Unable to create precomputed ChessMagics."),
            ),
        }
    }

    #[inline]
    pub fn new_board(&self) -> Board {
        Board::from(self.magics.clone())
    }

    #[inline]
    pub fn board_builder(&self) -> BoardBuilder {
        BoardBuilder::from(self.magics.clone())
    }

    #[inline]
    pub fn from_fen(&self, fen: &str) -> Result<Board, FenErr> {
        from_fen(fen, BoardBuilder::from(self.magics.clone()))
    }

    #[inline]
    pub fn moves_from_fen(&self, fen: &str) -> Result<Option<Vec<Piece<PieceMove>>>, FenErr> {
        self.from_fen(fen).map(|board| board.next_moves())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece::*;
    use crate::piece_move::PieceMove::*;
    use crate::sq;
    use lazy_static::lazy_static;
    use std::collections::HashSet;

    // Initialize RUKY only once here, since it's a convenient wrapper around the
    // magics.
    lazy_static! {
        static ref RUKY: Ruky = Ruky::new();
    }

    #[test]
    fn moves_from_fen1() {
        let moves = RUKY
            .moves_from_fen("8/1P6/8/1K2n3/6Pp/8/5P2/2k5 w - - 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Promo {
                from: sq::B7,
                to: sq::B8,
                promo: Knight(()),
            }),
            Pawn(Promo {
                from: sq::B7,
                to: sq::B8,
                promo: Bishop(()),
            }),
            Pawn(Promo {
                from: sq::B7,
                to: sq::B8,
                promo: Rook(()),
            }),
            Pawn(Promo {
                from: sq::B7,
                to: sq::B8,
                promo: Queen(()),
            }),
            Pawn(Simple {
                from: sq::F2,
                to: sq::F3,
            }),
            Pawn(Simple {
                from: sq::F2,
                to: sq::F4,
            }),
            Pawn(Simple {
                from: sq::G4,
                to: sq::G5,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::A4,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::A5,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::A6,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::B4,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::B6,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::C5,
            }),
        ]);
        assert_eq!(HashSet::from_iter(moves), expected_moves);
    }
}

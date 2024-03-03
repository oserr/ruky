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

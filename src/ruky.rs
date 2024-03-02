use crate::board::Board;
use crate::magics::ChessMagics;
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
}

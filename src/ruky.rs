use crate::board::{Board, BoardBuilder};
use crate::fen::{from_fen, FenErr};
use crate::magics::ChessMagics;
use crate::piece::Piece;
use crate::piece_move::PieceMove;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ruky {
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
    use crate::board::GameState;
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

    #[test]
    fn moves_from_fen2() {
        let moves = RUKY
            .moves_from_fen("8/8/7k/1K6/3b4/R1PP2n1/8/8 w - - 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::C3,
                to: sq::C4,
            }),
            Pawn(Capture {
                from: sq::C3,
                to: sq::D4,
                cap: Bishop(()),
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A1,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A2,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::B3,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A4,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A5,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A6,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A7,
            }),
            Rook(Simple {
                from: sq::A3,
                to: sq::A8,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::A4,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::B4,
            }),
            King(Simple {
                from: sq::B5,
                to: sq::C4,
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
                to: sq::C6,
            }),
        ]);
        assert_eq!(HashSet::from_iter(moves), expected_moves);
    }

    #[test]
    fn moves_from_fen3() {
        let moves = RUKY
            .moves_from_fen("2B2b1K/p7/8/n2Q1p2/8/8/3P4/5k2 w - - 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::D2,
                to: sq::D3,
            }),
            Pawn(Simple {
                from: sq::D2,
                to: sq::D4,
            }),
            Bishop(Simple {
                from: sq::C8,
                to: sq::B7,
            }),
            Bishop(Simple {
                from: sq::C8,
                to: sq::A6,
            }),
            Bishop(Simple {
                from: sq::C8,
                to: sq::D7,
            }),
            Bishop(Simple {
                from: sq::C8,
                to: sq::E6,
            }),
            Bishop(Capture {
                from: sq::C8,
                to: sq::F5,
                cap: Pawn(()),
            }),
            King(Simple {
                from: sq::H8,
                to: sq::G8,
            }),
            King(Simple {
                from: sq::H8,
                to: sq::H7,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::A2,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::B3,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::C4,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::E4,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::F3,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::G2,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::H1,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::D4,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::D3,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::D6,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::D7,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::D8,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::C5,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::B5,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::E5,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::G8,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::C6,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::B7,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::A8,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::E6,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::F7,
            }),
            Queen(Simple {
                from: sq::D5,
                to: sq::G8,
            }),
            Queen(Capture {
                from: sq::D5,
                to: sq::A5,
                cap: Knight(()),
            }),
            Queen(Capture {
                from: sq::D5,
                to: sq::F5,
                cap: Pawn(()),
            }),
        ]);
        assert_eq!(HashSet::from_iter(moves), expected_moves);
    }

    #[test]
    fn moves_from_fen4() {
        let moves = RUKY
            .moves_from_fen("r3k2r/1ppqbppp/p1np1n2/4p3/2B1PPb1/2NP1N2/PPPBQ1PP/R3K2R w KQkq - 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::A2,
                to: sq::A3,
            }),
            Pawn(Simple {
                from: sq::A2,
                to: sq::A4,
            }),
            Pawn(Simple {
                from: sq::B2,
                to: sq::B3,
            }),
            Pawn(Simple {
                from: sq::B2,
                to: sq::B4,
            }),
            Pawn(Simple {
                from: sq::D3,
                to: sq::D4,
            }),
            Pawn(Simple {
                from: sq::F4,
                to: sq::F5,
            }),
            Pawn(Capture {
                from: sq::F4,
                to: sq::E5,
                cap: Pawn(()),
            }),
            Pawn(Simple {
                from: sq::G2,
                to: sq::G3,
            }),
            Pawn(Simple {
                from: sq::H2,
                to: sq::H3,
            }),
            Pawn(Simple {
                from: sq::H2,
                to: sq::H4,
            }),
            Knight(Simple {
                from: sq::C3,
                to: sq::B1,
            }),
            Knight(Simple {
                from: sq::C3,
                to: sq::A4,
            }),
            Knight(Simple {
                from: sq::C3,
                to: sq::B5,
            }),
            Knight(Simple {
                from: sq::C3,
                to: sq::D5,
            }),
            Knight(Simple {
                from: sq::C3,
                to: sq::D1,
            }),
            Knight(Simple {
                from: sq::F3,
                to: sq::D4,
            }),
            Knight(Capture {
                from: sq::F3,
                to: sq::E5,
                cap: Pawn(()),
            }),
            Knight(Simple {
                from: sq::F3,
                to: sq::G5,
            }),
            Knight(Simple {
                from: sq::F3,
                to: sq::H4,
            }),
            Knight(Simple {
                from: sq::F3,
                to: sq::G1,
            }),
            Bishop(Simple {
                from: sq::C4,
                to: sq::B3,
            }),
            Bishop(Simple {
                from: sq::C4,
                to: sq::B5,
            }),
            Bishop(Capture {
                from: sq::C4,
                to: sq::A6,
                cap: Pawn(()),
            }),
            Bishop(Simple {
                from: sq::C4,
                to: sq::D5,
            }),
            Bishop(Simple {
                from: sq::C4,
                to: sq::E6,
            }),
            Bishop(Capture {
                from: sq::C4,
                to: sq::F7,
                cap: Pawn(()),
            }),
            Bishop(Simple {
                from: sq::D2,
                to: sq::C1,
            }),
            Bishop(Simple {
                from: sq::D2,
                to: sq::E3,
            }),
            Rook(Simple {
                from: sq::A1,
                to: sq::B1,
            }),
            Rook(Simple {
                from: sq::A1,
                to: sq::C1,
            }),
            Rook(Simple {
                from: sq::A1,
                to: sq::D1,
            }),
            Rook(Simple {
                from: sq::H1,
                to: sq::F1,
            }),
            Rook(Simple {
                from: sq::H1,
                to: sq::G1,
            }),
            Queen(Simple {
                from: sq::E2,
                to: sq::D1,
            }),
            Queen(Simple {
                from: sq::E2,
                to: sq::E3,
            }),
            Queen(Simple {
                from: sq::E2,
                to: sq::F2,
            }),
            Queen(Simple {
                from: sq::E2,
                to: sq::F1,
            }),
            King(Simple {
                from: sq::E1,
                to: sq::D1,
            }),
            King(Simple {
                from: sq::E1,
                to: sq::F1,
            }),
            King(Simple {
                from: sq::E1,
                to: sq::F2,
            }),
            King(Castle {
                king_from: sq::E1,
                king_to: sq::G1,
                rook_from: sq::H1,
                rook_to: sq::F1,
            }),
            King(Castle {
                king_from: sq::E1,
                king_to: sq::C1,
                rook_from: sq::A1,
                rook_to: sq::D1,
            }),
        ]);

        let actual = HashSet::from_iter(moves);
        assert_eq!(actual, expected_moves);
    }

    #[test]
    fn moves_from_fen5() {
        let moves = RUKY
            .moves_from_fen("r3k2r/1ppqbppp/p1np1n2/4p3/2B1PPb1/2NP1N2/PPPBQ1PP/2KR3R b kq - 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::A6,
                to: sq::A5,
            }),
            Pawn(Simple {
                from: sq::B7,
                to: sq::B6,
            }),
            Pawn(Simple {
                from: sq::B7,
                to: sq::B5,
            }),
            Pawn(Simple {
                from: sq::D6,
                to: sq::D5,
            }),
            Pawn(Capture {
                from: sq::E5,
                to: sq::F4,
                cap: Pawn(()),
            }),
            Pawn(Simple {
                from: sq::G7,
                to: sq::G6,
            }),
            Pawn(Simple {
                from: sq::G7,
                to: sq::G5,
            }),
            Pawn(Simple {
                from: sq::H7,
                to: sq::H6,
            }),
            Pawn(Simple {
                from: sq::H7,
                to: sq::H5,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::D8,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::B8,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::A7,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::A5,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::B4,
            }),
            Knight(Simple {
                from: sq::C6,
                to: sq::D4,
            }),
            Knight(Simple {
                from: sq::F6,
                to: sq::D5,
            }),
            Knight(Capture {
                from: sq::F6,
                to: sq::E4,
                cap: Pawn(()),
            }),
            Knight(Simple {
                from: sq::F6,
                to: sq::H5,
            }),
            Knight(Simple {
                from: sq::F6,
                to: sq::G8,
            }),
            Bishop(Simple {
                from: sq::E7,
                to: sq::D8,
            }),
            Bishop(Simple {
                from: sq::E7,
                to: sq::F8,
            }),
            Bishop(Capture {
                from: sq::G4,
                to: sq::F3,
                cap: Knight(()),
            }),
            Bishop(Simple {
                from: sq::G4,
                to: sq::H3,
            }),
            Bishop(Simple {
                from: sq::G4,
                to: sq::H5,
            }),
            Bishop(Simple {
                from: sq::G4,
                to: sq::F5,
            }),
            Bishop(Simple {
                from: sq::G4,
                to: sq::E6,
            }),
            Rook(Simple {
                from: sq::A8,
                to: sq::A7,
            }),
            Rook(Simple {
                from: sq::A8,
                to: sq::B8,
            }),
            Rook(Simple {
                from: sq::A8,
                to: sq::C8,
            }),
            Rook(Simple {
                from: sq::A8,
                to: sq::D8,
            }),
            Rook(Simple {
                from: sq::H8,
                to: sq::F8,
            }),
            Rook(Simple {
                from: sq::H8,
                to: sq::G8,
            }),
            Queen(Simple {
                from: sq::D7,
                to: sq::C8,
            }),
            Queen(Simple {
                from: sq::D7,
                to: sq::D8,
            }),
            Queen(Simple {
                from: sq::D7,
                to: sq::E6,
            }),
            Queen(Simple {
                from: sq::D7,
                to: sq::F5,
            }),
            King(Simple {
                from: sq::E8,
                to: sq::D8,
            }),
            King(Simple {
                from: sq::E8,
                to: sq::F8,
            }),
            King(Castle {
                king_from: sq::E8,
                king_to: sq::G8,
                rook_from: sq::H8,
                rook_to: sq::F8,
            }),
            King(Castle {
                king_from: sq::E8,
                king_to: sq::C8,
                rook_from: sq::A8,
                rook_to: sq::D8,
            }),
        ]);

        let actual = HashSet::from_iter(moves);
        assert_eq!(actual, expected_moves);
    }

    #[test]
    fn moves_from_fen6() {
        let moves = RUKY
            .moves_from_fen("3k4/2p5/8/2Pp4/8/8/4K3/8 w - d6 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::C5,
                to: sq::C6,
            }),
            Pawn(EnPassant {
                from: sq::C5,
                to: sq::D6,
                passant: sq::D5,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::E1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D2,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F2,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D3,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::E3,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F3,
            }),
        ]);

        let actual = HashSet::from_iter(moves);
        assert_eq!(actual, expected_moves);
    }

    #[test]
    fn moves_from_fen7() {
        let moves = RUKY
            .moves_from_fen("3k4/2p5/8/1pP5/8/8/4K3/8 w - b6 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::C5,
                to: sq::C6,
            }),
            Pawn(EnPassant {
                from: sq::C5,
                to: sq::B6,
                passant: sq::B5,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::E1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F1,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D2,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F2,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::D3,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::E3,
            }),
            King(Simple {
                from: sq::E2,
                to: sq::F3,
            }),
        ]);

        let actual = HashSet::from_iter(moves);
        assert_eq!(actual, expected_moves);
    }

    #[test]
    fn moves_from_fen8() {
        let moves = RUKY
            .moves_from_fen("8/8/8/3k4/2pP4/8/2P5/2K5 b - d3 0 1")
            .expect("Fen is OK")
            .expect("Moves are OK");

        let expected_moves = HashSet::from([
            Pawn(Simple {
                from: sq::C4,
                to: sq::C3,
            }),
            Pawn(EnPassant {
                from: sq::C4,
                to: sq::D3,
                passant: sq::D4,
            }),
            King(Simple {
                from: sq::D5,
                to: sq::C6,
            }),
            King(Simple {
                from: sq::D5,
                to: sq::D6,
            }),
            King(Simple {
                from: sq::D5,
                to: sq::E6,
            }),
            King(Simple {
                from: sq::D5,
                to: sq::E4,
            }),
            King(Capture {
                from: sq::D5,
                to: sq::D4,
                cap: Pawn(()),
            }),
        ]);

        let actual = HashSet::from_iter(moves);
        assert_eq!(actual, expected_moves);
    }

    #[test]
    fn terminal_state_from_fen() {
        let board = RUKY
            .from_fen("2Kn4/7r/2n5/3p1p2/1b1p1p2/3r4/4k3/8 w - - 4 74")
            .expect("FEN is OK");
        assert!(board.is_terminal());
        assert!(!board.is_check());
        assert_eq!(board.game_state(), GameState::Draw);
    }
}

use crate::piece::Piece;
use crate::sq::Sq;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PieceMove {
    Simple {
        from: Sq,
        to: Sq,
    },
    Capture {
        from: Sq,
        to: Piece<Sq>,
    },
    Castle {
        king_from: Sq,
        king_to: Sq,
        rook_from: Sq,
        rook_to: Sq,
    },
    EnPassant {
        from: Sq,
        to: Sq,
        passant: Sq,
    },
    Promo {
        from: Sq,
        to: Sq,
    },
    PromoCap {
        from: Sq,
        to: Piece<Sq>,
    },
}

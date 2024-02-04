use crate::piece::Piece;
use crate::sq::Sq;

// An enum for representing the different types of moves for all chess pieces.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PieceMove {
    // Simple represents simple moves that only need a source and destination square to be fully
    // specified.
    Simple {
        from: Sq,
        to: Sq,
    },

    // For simple captures where the captured piece is on the destination square.
    Capture {
        from: Sq,
        to: Piece<Sq>,
    },

    // Castling between king and rook, the only type of move where two pieces move simultaneously.
    Castle {
        king_from: Sq,
        king_to: Sq,
        rook_from: Sq,
        rook_to: Sq,
    },

    // Capture by en-passant.
    EnPassant {
        from: Sq,
        to: Sq,
        // The en-passant square, where the enemy pawn is located.
        passant: Sq,
    },

    // Pawn promotion.
    Promo {
        from: Sq,
        to: Sq,
        // The piece to which the pawn is promoted.
        promo: Piece<()>,
    },

    // Pawn promotion with capture.
    PromoCap {
        from: Sq,
        // Destination square with captured piece.
        to: Piece<Sq>,
        // The piece to which the pawn is promoted.
        promo: Piece<()>,
    },
}

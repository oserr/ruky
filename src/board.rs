use crate::bitboard::{BitBoard, RANK_3, RANK_6};
use crate::magics::ChessMagics;
use crate::piece::{Color, Piece, Piece::*};
use crate::piece_move::{PieceMove, PieceMove::*};
use crate::piece_set::{AttackSquares, PieceSet};
use crate::sq::Sq;
use std::sync::Arc;

// Represents a chess board, and encodes the rules for moving pieces and
// determining the current game state, e.g. whether the game is drawn.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Board {
    // The board state. We use a Box for it because this makes it much cheaper to move a board.
    state: Box<BoardState>,

    // We use an Arc for ChessMagics, because ChessMagics are expensive to compute, and hence we
    // want to share one instance of chess magics where ever they are needed, and between threads.
    magics: Arc<ChessMagics>,
}

impl Board {
    // Computes all the moves, including moves that are not legal, e.g. putting
    // oneself in check. If there are no moves to be made, e.g. we're already in
    // a terminal state, then it return None.
    pub fn all_moves(&self) -> Option<Vec<Piece<PieceMove>>> {
        if self.state.game_state.is_terminal() {
            return None;
        }

        let mut moves: Vec<Piece<PieceMove>> = Vec::new();

        self.king_moves(&mut moves);
        self.queen_moves(&mut moves);
        self.rook_moves(&mut moves);
        self.bishop_moves(&mut moves);
        self.knight_moves(&mut moves);
        self.pawn_moves(&mut moves);

        if moves.is_empty() {
            None
        } else {
            Some(moves)
        }
    }

    fn king_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(King(self.state.mine.king()), moves, |b| b.king_moves());
        let (king_castle, queen_castle) = self
            .state
            .mine
            .castle(&self.state.other, self.state.other_attacks.all());
        if king_castle.is_some() {
            moves.push(king_castle.unwrap());
        }
        if queen_castle.is_some() {
            moves.push(queen_castle.unwrap());
        }
    }

    fn queen_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Queen(self.state.mine.queens()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .qmagics(from, self.state.all())
                .expect("Unable to to compute queen magics")
        });
    }

    fn rook_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Rook(self.state.mine.rooks()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .rmagics(from, self.state.all())
                .expect("Unable to to compute rook magics")
        });
    }

    fn bishop_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Bishop(self.state.mine.bishops()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .bmagics(from, self.state.all())
                .expect("Unable to to compute bishop magics")
        });
    }

    fn knight_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Knight(self.state.mine.knights()), moves, |b| {
            b.knight_moves()
        });
    }

    fn pawn_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        if self.state.color().is_white() {
            self.add_pawn_moves(
                moves,
                |bits, empty| bits.wp_moves(empty),
                |s| s.in_last_rank(),
            );
        } else {
            self.add_pawn_moves(
                moves,
                |bits, empty| bits.bp_moves(empty),
                |s| s.in_first_rank(),
            );
        }
    }

    fn add_pawn_moves(
        &self,
        moves: &mut Vec<Piece<PieceMove>>,
        moves_fn: impl Fn(BitBoard, BitBoard) -> (BitBoard, BitBoard),
        is_promo: impl Fn(Sq) -> bool,
    ) {
        let pawns = self.state.mine.pawns();
        let empty = self.state.none();
        let other = self.state.other.all();

        for (from, pawn_bit) in pawns.sq_bit_iter() {
            let (forward_moves, all_attacks) = moves_fn(pawn_bit, empty);

            for to in forward_moves.sq_iter() {
                if is_promo(to) {
                    add_promo(from, to, moves);
                } else {
                    moves.push(Pawn(Simple { from, to }));
                }
            }

            let attacks = all_attacks & other;

            for to in attacks.sq_iter() {
                let cap = self
                    .state
                    .other
                    .find_type(to)
                    .expect("Unable to find a piece for capture.");

                if is_promo(to) {
                    add_promo_with_cap(from, to, cap, moves);
                } else {
                    moves.push(Pawn(Capture { from, to, cap }));
                }
            }

            if let Some(passant_cap) = self
                .state
                .passant_sq
                .and_then(|ps| ps.by_enpassant(from, all_attacks))
            {
                moves.push(passant_cap)
            }
        }
    }

    fn simple_moves(
        &self,
        piece: Piece<BitBoard>,
        moves: &mut Vec<Piece<PieceMove>>,
        move_fn: impl Fn(BitBoard) -> BitBoard,
    ) {
        for (from, bit) in piece.val().sq_bit_iter() {
            let bit_moves = move_fn(bit);

            let non_attacks = bit_moves & self.state.none();
            for to in non_attacks.sq_iter() {
                moves.push(piece.with(Simple { from, to }));
            }

            let attacks = bit_moves & self.state.other.all();
            for to in attacks.sq_iter() {
                let cap = self
                    .state
                    .other
                    .find_type(to)
                    .expect("Unable to find an attack piece.");
                moves.push(piece.with(Capture { from, to, cap }));
            }
        }
    }
}

// Converts ChessMagics into a Board.
impl From<Arc<ChessMagics>> for Board {
    fn from(magics: Arc<ChessMagics>) -> Board {
        let state = Box::new(BoardState::default());
        Board { state, magics }
    }
}

// BoardState holds all the state needed needed to a play a game of regular
// chess, including the position of the pieces, position of squares that are
// attacked, the current game state, the number of half moves, the number of
// full moves, and whether there is an opportunity to capture by en passant.
// Note that castling rights are encoded in the PieceSets.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BoardState {
    // The pieces that are moving next. We use a Box for the pieces other because this
    // makes it much cheaper to swap the pieces after a move is made. This simplifies a lot of code
    // because we can do things in terms of the player moving next.
    mine: Box<PieceSet>,

    // The pieces that are not moving next.
    other: Box<PieceSet>,

    // All positions on the board that are attacked by the pieces moving next.
    my_attacks: AttackSquares,

    // All positions on the board that are attacked by the pieces not moving.
    other_attacks: AttackSquares,

    // The current game state.
    game_state: GameState,

    // The number of half moves, which is used to determine if a game should be drawn because of
    // insufficient progress, i.e. no pawn moves or captures.
    half_move: u16,

    // The number of full moves. In theory, you can have an infinite number of moves in a game, but
    // in practice the game is drawn at some point of there is no progress.
    full_move: u16,

    // If set, represents the square where capture by en-passant is possible.
    passant_sq: Option<PassantSq>,
}

impl BoardState {
    // Returns the union of all pieces as a BitBoard.
    fn all(&self) -> BitBoard {
        self.mine.all() | self.other.all()
    }

    // Returns the set of all empty squares.
    fn none(&self) -> BitBoard {
        !self.all()
    }

    // Returns the color moving next.
    fn color(&self) -> Color {
        self.mine.color()
    }
}

// Initializes the BoardState for a new game.
impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            mine: Box::new(PieceSet::init_white()),
            other: Box::new(PieceSet::init_black()),
            my_attacks: AttackSquares {
                pieces: BitBoard::new(),
                no_pieces: RANK_3,
            },
            other_attacks: AttackSquares {
                pieces: BitBoard::new(),
                no_pieces: RANK_6,
            },
            game_state: GameState::Next(Color::White),
            half_move: 0,
            full_move: 0,
            passant_sq: None,
        }
    }
}

// Represents the current game state. Mate and Draw are final game state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameState {
    Next(Color),
    Check(Color),
    Mate(Color),
    Draw,
}

impl GameState {
    // Returns true if this GameState reprsesents a terminal game state, i.e. a draw
    // or check mate.
    pub fn is_terminal(&self) -> bool {
        match *self {
            GameState::Next(_) | GameState::Check(_) => false,
            _ => true,
        }
    }
}

// A struct to represent the position where en-passant capture is possible.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PassantSq {
    // Represents the actual location of the piece. This is a square in either the 4th or 5th rank.
    actual: Sq,

    // Represents the square where the pawn is captured. This is a square in either the 3rd or 6th
    // rank.
    capture: Sq,
}

impl PassantSq {
    fn by_enpassant(&self, from: Sq, attacks: BitBoard) -> Option<Piece<PieceMove>> {
        if attacks.has_bit(self.capture) {
            Some(Pawn(EnPassant {
                from,
                to: self.capture,
                passant: self.actual,
            }))
        } else {
            None
        }
    }
}

// Utility to add all the different types of promotions.
fn add_promo(from: Sq, to: Sq, moves: &mut Vec<Piece<PieceMove>>) {
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Queen(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Rook(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Bishop(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Knight(()),
    }));
}

// Utility to add all the different types of promotions.
fn add_promo_with_cap(from: Sq, to: Sq, cap: Piece<()>, moves: &mut Vec<Piece<PieceMove>>) {
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Queen(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Rook(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Bishop(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Knight(()),
        cap,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref MAGICS: Arc<ChessMagics> = Arc::new(
            ChessMagics::from_precomputed().expect("Unable to compute magics for unit test.")
        );
    }

    #[test]
    fn board_init_from_magics() {
        let board = Board::from(MAGICS.clone());

        assert_eq!(*board.state.mine, PieceSet::init_white());
        assert_eq!(*board.state.other, PieceSet::init_black());
        assert_eq!(board.state.game_state, GameState::Next(Color::White));
        assert_eq!(board.state.half_move, 0);
        assert_eq!(board.state.full_move, 0);
        assert_eq!(board.state.passant_sq, None);
    }
}

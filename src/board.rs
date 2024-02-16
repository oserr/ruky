use crate::bitboard::{BitBoard, RANK_3, RANK_6};
use crate::magics::ChessMagics;
use crate::piece::{Color, Piece, Piece::*};
use crate::piece_move::{PieceMove, PieceMove::*};
use crate::piece_set::{AttackSquares, PieceSet};
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
    fn king_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(King(self.state.mine.king()), moves, |b| b.king_moves());
    }

    fn rook_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Rook(self.state.mine.rooks()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .rmagics(from, self.state.all())
                .expect("Unable to to compute rook magics")
        });
    }

    fn knight_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Knight(self.state.mine.knights()), moves, |b| {
            b.knight_moves()
        });
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

    // If set, represents the file where capture by en-passant is possible.
    passant_file: Option<u8>,
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
            passant_file: None,
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

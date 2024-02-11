use crate::magics::ChessMagics;
use crate::piece::Color;
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

// Converts ChessMagics into a Board.
impl From<Arc<ChessMagics>> for Board {
    fn from(magics: Arc<ChessMagics>) -> Board {
        let state = Box::new(BoardState::from(magics.as_ref()));
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

// Initializes the BoardState for a new game with ChessMagics.
impl From<&ChessMagics> for BoardState {
    fn from(magics: &ChessMagics) -> BoardState {
        let white = Box::new(PieceSet::init_white());
        let black = Box::new(PieceSet::init_black());

        // TODO: we don't really need magics here to determine the attack squares.
        let white_attacks = white.attacks(&black, magics);
        let black_attacks = black.attacks(&white, magics);

        BoardState {
            mine: white,
            other: black,
            my_attacks: white_attacks,
            other_attacks: black_attacks,
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

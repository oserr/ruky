use crate::magics::ChessMagics;
use crate::piece::Color;
use crate::piece_set::{AttackSquares, PieceSet};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Board {
    state: Box<BoardState>,
    magics: Arc<ChessMagics>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BoardState {
    mine: Box<PieceSet>,
    other: Box<PieceSet>,
    my_attacks: AttackSquares,
    other_attacks: AttackSquares,
    game_state: GameState,
    half_move: u16,
    full_move: u16,
    passant_file: Option<u8>,
    wk_castle: bool,
    wq_castle: bool,
    bk_castle: bool,
    bq_castle: bool,
}

impl From<&ChessMagics> for BoardState {
    fn from(magics: &ChessMagics) -> BoardState {
        let white = Box::new(PieceSet::init_white());
        let black = Box::new(PieceSet::init_black());
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
            wk_castle: true,
            wq_castle: true,
            bk_castle: true,
            bq_castle: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameState {
    Next(Color),
    Check(Color),
    Mate(Color),
    Draw,
}

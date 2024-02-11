use crate::bitboard::BitBoard;
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameState {
    Next(Color),
    Check(Color),
    Mate(Color),
    Draw,
}

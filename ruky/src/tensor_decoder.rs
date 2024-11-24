// This module contains components for decoding the policy and value tensors
// into boards and moves.

use crate::board::Board;
use crate::err::RukyErr;
use crate::piece::Piece;
use crate::piece_move::PieceMove;
use burn::prelude::{Backend, Tensor};
use std::marker::PhantomData;

// A structure representing the decoded board states, their probabilities, and
// the value of the current board state.
#[derive(Clone, Debug)]
pub(crate) struct DecBoards {
    // The collection of Board states decoded from a tensor, with a corresponding probability that
    // this is the best move given the current position.
    board_probs: Vec<(Board, f32)>,

    // The value of the current board state.
    value: f32,
}

// Same as DecBoards, but contains the previous board, a collection of the moves
// and the probabilities.
#[derive(Clone, Debug)]
pub(crate) struct DecMoves {
    prev_board: Board,
    move_probs: Vec<(Piece<PieceMove>, f32)>,
    value: f32,
}

pub trait TensorDecoder<B: Backend> {
    fn decode_boards(
        board: &Board,
        mv_tensor: Tensor<B, 4>,
        eval_tensor: Tensor<B, 4>,
    ) -> Result<DecBoards, RukyErr>;
    fn decode_moves(
        board: &Board,
        mv_tensor: Tensor<B, 4>,
        eval_tensor: Tensor<B, 4>,
    ) -> Result<DecMoves, RukyErr>;
}

#[derive(Clone, Debug)]
struct AzDecoder<B: Backend> {
    _backend: PhantomData<B>,
}

impl<B: Backend> TensorDecoder<B> for AzDecoder<B> {
    fn decode_boards(
        _board: &Board,
        _mv_tensor: Tensor<B, 4>,
        _eval_tensor: Tensor<B, 4>,
    ) -> Result<DecBoards, RukyErr> {
        todo!();
    }

    fn decode_moves(
        _board: &Board,
        _mv_tensor: Tensor<B, 4>,
        _eval_tensor: Tensor<B, 4>,
    ) -> Result<DecMoves, RukyErr> {
        todo!();
    }
}

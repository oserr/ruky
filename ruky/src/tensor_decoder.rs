// This module contains components for decoding the policy and value tensors
// into boards and moves.

use crate::board::Board;
use crate::ecmv::EcMove;
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
pub(crate) struct AzDecoder<B: Backend> {
    _backend: PhantomData<B>,
}

impl<B: Backend> TensorDecoder<B> for AzDecoder<B> {
    fn decode_boards(
        board: &Board,
        mv_tensor: Tensor<B, 4>,
        eval_tensor: Tensor<B, 4>,
    ) -> Result<DecBoards, RukyErr> {
        let mv_tensor_data = mv_tensor.to_data();
        let mv_data = mv_tensor_data
            .as_slice::<f32>()
            .map_err(|_| RukyErr::InputIsNotValid)?;
        if mv_data.len() != N_POSSIBLE_MOVES {
            return Err(RukyErr::MoveTensorDim);
        }

        let mut total = 0.0;
        let mut board_probs = Vec::<(Board, f32)>::new();

        for next_board in board.next_boards().ok_or(RukyErr::NoMovesButExpected)? {
            let last_move = next_board
                .last_move()
                .expect("Board should have a last move.");
            let index = EcMove::from(last_move).index();
            let prob = mv_data[index].exp();
            total += prob;
            board_probs.push((next_board, prob));
        }

        board_probs.iter_mut().for_each(|(_, prob)| *prob /= total);

        Ok(DecBoards {
            board_probs,
            value: get_value(&eval_tensor)?,
        })
    }

    fn decode_moves(
        board: &Board,
        mv_tensor: Tensor<B, 4>,
        eval_tensor: Tensor<B, 4>,
    ) -> Result<DecMoves, RukyErr> {
        let mv_tensor_data = mv_tensor.to_data();
        let mv_data = mv_tensor_data
            .as_slice::<f32>()
            .map_err(|_| RukyErr::InputIsNotValid)?;
        if mv_data.len() != N_POSSIBLE_MOVES {
            return Err(RukyErr::MoveTensorDim);
        }

        let mut total = 0.0;
        let mut move_probs = Vec::<(Piece<PieceMove>, f32)>::new();

        for next_move in board.next_moves().ok_or(RukyErr::NoMovesButExpected)? {
            let index = EcMove::from(next_move).index();
            let prob = mv_data[index].exp();
            total += prob;
            move_probs.push((next_move, prob));
        }

        move_probs.iter_mut().for_each(|(_, prob)| *prob /= total);

        Ok(DecMoves {
            prev_board: board.clone(),
            move_probs,
            value: get_value(&eval_tensor)?,
        })
    }
}

fn get_value<B: Backend>(tensor: &Tensor<B, 4>) -> Result<f32, RukyErr> {
    let eval_tensor_data = tensor.to_data();
    let eval_data = eval_tensor_data
        .as_slice()
        .map_err(|_| RukyErr::InputIsNotValid)?;
    if eval_data.len() != 1 {
        return Err(RukyErr::EvalTensorDim);
    }
    Ok(eval_data[0])
}

const N_POSSIBLE_MOVES: usize = 73 * 8 * 8;

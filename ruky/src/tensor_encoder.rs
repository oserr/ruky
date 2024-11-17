// This module contains components for encoding boards and moves to encoded
// tensors.

use crate::board::Board;
use crate::search::{Bp, Mp};
use burn::prelude::{Backend, Tensor};

trait TensorEncoder<B: Backend> {
    fn encode_board(board: &Board) -> Tensor<B, 4>;
    fn encode_boards(boards: &[Board]) -> Tensor<B, 4>;
    fn encode_mps(mps: &[Mp]) -> Tensor<B, 4>;
    fn encode_bps(bps: &[Bp]) -> Tensor<B, 4>;
}

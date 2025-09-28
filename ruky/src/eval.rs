// This module contains components for evaluating a position.

use crate::board::Board;
use crate::err::RukyErr;
use crate::nn::AlphaZeroNet;
use crate::tensor_decoder::{AzDecoder, DecBoards, TensorDecoder};
use crate::tensor_encoder::{AzEncoder, TensorEncoder};
use burn::{prelude::Backend, tensor::activation::softmax};
use std::sync::Arc;

// Re-use DecBoards and DecMoves as the objects returned by the Eval trait, but
// give them names more appropriate for the use.
pub type EvalBoards = DecBoards;

// A trait for evaluting a board position.
pub trait Eval {
    fn eval(&self, board: &Board) -> Result<EvalBoards, RukyErr>;
    fn eval_boards(&self, boards: &[Board]) -> Result<EvalBoards, RukyErr>;
    fn eval_batch_data(
        &self,
        batch_size: usize,
        data: Vec<f32>,
    ) -> Result<(Vec<f32>, Vec<f32>), RukyErr>;
}

pub struct AzEval<B: Backend> {
    encoder: AzEncoder<B>,
    decoder: AzDecoder<B>,
    net: Arc<AlphaZeroNet<B>>,
}

impl<B: Backend> Eval for AzEval<B> {
    fn eval(&self, board: &Board) -> Result<EvalBoards, RukyErr> {
        let input = self.encoder.encode_board(board);
        let (mv_tensor, eval_tensor) = self.net.forward(input);
        self.decoder.decode_boards(board, mv_tensor, eval_tensor)
    }

    fn eval_boards(&self, boards: &[Board]) -> Result<EvalBoards, RukyErr> {
        let input = self.encoder.encode_boards(boards);
        let (mv_tensor, eval_tensor) = self.net.forward(input);
        let board = boards.last().expect("Expecting at least 1 board for eval.");
        self.decoder.decode_boards(board, mv_tensor, eval_tensor)
    }

    fn eval_batch_data(
        &self,
        batch_size: usize,
        data: Vec<f32>,
    ) -> Result<(Vec<f32>, Vec<f32>), RukyErr> {
        let input = self.encoder.encode_batch_data(batch_size, data);
        let (mv_tensor, eval_tensor) = self.net.forward(input);
        let mv_tensor = softmax(mv_tensor, 3);
        let mv_data = mv_tensor
            .into_data()
            .into_vec()
            .expect("Expecing data from move tensor.");
        let eval_data = eval_tensor
            .into_data()
            .into_vec()
            .expect("Expecting data from eval tensor.");
        Ok((mv_data, eval_data))
    }
}

impl<B: Backend> AzEval<B> {
    pub fn create(encoder: AzEncoder<B>, decoder: AzDecoder<B>, net: Arc<AlphaZeroNet<B>>) -> Self {
        Self {
            encoder,
            decoder,
            net,
        }
    }
}

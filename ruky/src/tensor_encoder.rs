// This module contains components for encoding boards and moves to tensors.

use crate::board::Board;
use crate::piece_set::PieceSet;
use crate::search::{Bp, Mp};
use burn::prelude::{Backend, Tensor, TensorData};

trait TensorEncoder<B: Backend> {
    fn encode_board(&self, board: &Board) -> Tensor<B, 4>;
    fn encode_boards(&self, boards: &[Board]) -> Tensor<B, 4>;
    fn encode_mps(&self, mps: &[Mp]) -> Tensor<B, 4>;
    fn encode_bps(&self, bps: &[Bp]) -> Tensor<B, 4>;
}

// AzEncoder represents the AlphaZero encoder, i.e. it encodes the board the
// board state for input into the AlphaZero network. For a given board position,
// it outputs a 119 x 8 x 8 tensor. 6 planes are used to represent one set of
// the pieces, 6 for the other set of pieces, and 2 planes for repetition count
// in current position for each player. Hence, the current board position uses
// 14 total planes, but this is repeated for 8 total time steps. If there are no
// previous positions, the planes are set to zeros. The last 7 planes are used
// to represent
// - 1 for the current color
// - 1 for the total move count
// - 1 for king castling for the current player
// - 1 for queenside castling for the current player
// - 1 for king castling for the other player
// - 1 for queen castling for the other player
// - 1 for the progress count (i.e. 50 move rule)
struct AzEncoder<B: Backend> {
    device: B::Device,
}

impl<B: Backend> TensorEncoder<B> for AzEncoder<B> {
    fn encode_board(&self, _board: &Board) -> Tensor<B, 4> {
        todo!();
    }

    fn encode_boards(&self, boards: &[Board]) -> Tensor<B, 4> {
        assert!(!boards.is_empty());
        let mut data = vec![0.0; 119 * 64];
        for (board, chunk) in std::iter::zip(
            boards.into_iter().rev().take(8),
            data.chunks_exact_mut(64 * 14),
        ) {
            let finish_index = 6 * 64;
            encode_pieces(board.white(), &mut chunk[..finish_index]);
            encode_pieces(board.black(), &mut chunk[finish_index..2 * finish_index]);
            // TODO: need to set the repetition count on the last two planes.
        }
        let tensor_data = TensorData::new(data, [1, 119, 8, 8]);

        // TODO: finish setting up the rest of the state.
        Tensor::from_data(tensor_data, &self.device)
    }

    fn encode_mps(&self, _mps: &[Mp]) -> Tensor<B, 4> {
        todo!();
    }

    fn encode_bps(&self, _bps: &[Bp]) -> Tensor<B, 4> {
        todo!();
    }
}

fn encode_pieces(pieces: &PieceSet, data: &mut [f32]) {
    assert!(data.len() == 6 * 64);
    for (piece, chunk) in std::iter::zip(pieces.iter(), data.chunks_exact_mut(64)) {
        for sq in piece.val().sq_iter() {
            chunk[sq.as_usize()] = 1.0;
        }
    }
}

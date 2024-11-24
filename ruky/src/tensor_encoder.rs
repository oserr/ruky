// This module contains components for encoding boards and moves to tensors.

use crate::board::Board;
use crate::ecmv::EcMove;
use crate::piece_set::PieceSet;
use crate::search::{Bp, Mp};
use burn::prelude::{Backend, Tensor, TensorData};
use std::iter::zip;

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
    fn encode_board(&self, board: &Board) -> Tensor<B, 4> {
        let mut data = vec![0.0; 119 * 64];
        let six_planes = 6 * 64;

        // TODO: For black, might want to flip the board so it's from the player's
        // perspective.
        let (next_to_play, after_to_play) = if board.is_white_next() {
            (board.white(), board.black())
        } else {
            (board.black(), board.white())
        };

        encode_pieces(next_to_play, &mut data[..six_planes]);
        encode_pieces(after_to_play, &mut data[six_planes..2 * six_planes]);
        // TODO: need to set the repetition count on the last two planes.

        let state_features = get_state_features(&board);
        for (val, chunk) in zip(state_features.iter().rev(), data.rchunks_exact_mut(64)) {
            chunk.fill(*val);
        }

        let tensor_data = TensorData::new(data, [1, 119, 8, 8]);
        Tensor::from_data(tensor_data, &self.device)
    }

    fn encode_boards(&self, boards: &[Board]) -> Tensor<B, 4> {
        assert!(!boards.is_empty());

        let mut data = vec![0.0; 119 * 64];
        let six_planes = 6 * 64;

        for (board, chunk) in zip(
            boards.into_iter().rev().take(8),
            data.chunks_exact_mut(64 * 14),
        ) {
            // TODO: For black, might want to flip the board so it's from the player's
            // perspective.
            let (next_to_play, after_to_play) = if board.is_white_next() {
                (board.white(), board.black())
            } else {
                (board.black(), board.white())
            };

            encode_pieces(next_to_play, &mut chunk[..six_planes]);
            encode_pieces(after_to_play, &mut chunk[six_planes..2 * six_planes]);
            // TODO: need to set the repetition count on the last two planes.
        }

        let board = boards
            .last()
            .expect("boards should have at least one board.");
        let state_features = get_state_features(&board);

        for (val, chunk) in zip(state_features.iter().rev(), data.rchunks_exact_mut(64)) {
            chunk.fill(*val);
        }

        let tensor_data = TensorData::new(data, [1, 119, 8, 8]);
        Tensor::from_data(tensor_data, &self.device)
    }

    // Encodes the move probabilities in mps as a tensor.
    fn encode_mps(&self, mps: &[Mp]) -> Tensor<B, 4> {
        assert!(!mps.is_empty());
        let mut data = vec![0.0; 73 * 64];
        let total_visits = mps.iter().fold(0, |acc, mp| acc + mp.visits) as f32;
        for mp in mps {
            let ec_move = EcMove::from(mp.pm);
            let index = ec_move.index();
            data[index] = mp.visits as f32 / total_visits;
        }
        let tensor_data = TensorData::new(data, [1, 73, 8, 8]);
        Tensor::from_data(tensor_data, &self.device)
    }

    // Encodes the move probabilities in bps as a tensor.
    fn encode_bps(&self, bps: &[Bp]) -> Tensor<B, 4> {
        assert!(!bps.is_empty());
        let mut data = vec![0.0; 73 * 64];
        let total_visits = bps.iter().fold(0, |acc, bp| acc + bp.visits) as f32;
        for bp in bps {
            let ec_move = EcMove::from(bp.last_move());
            let index = ec_move.index();
            data[index] = bp.visits as f32 / total_visits;
        }
        let tensor_data = TensorData::new(data, [1, 73, 8, 8]);
        Tensor::from_data(tensor_data, &self.device)
    }
}

fn get_state_features(board: &Board) -> [f32; 7] {
    [
        board.is_white_next().into(),
        board.full_moves() as f32,
        board.has_wk_castle().into(),
        board.has_wq_castle().into(),
        board.has_bk_castle().into(),
        board.has_bq_castle().into(),
        board.half_moves().into(),
    ]
}

fn encode_pieces(pieces: &PieceSet, data: &mut [f32]) {
    assert!(data.len() == 6 * 64);
    for (piece, chunk) in zip(pieces.iter(), data.chunks_exact_mut(64)) {
        for sq in piece.val().sq_iter() {
            chunk[sq.as_usize()] = 1.0;
        }
    }
}

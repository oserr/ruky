// This module contains components for encoding boards and moves to tensors.

use crate::board::Board;
use crate::ecmv::EcMove;
use crate::piece_set::PieceSet;
use crate::search::{Bp, Mp};
use burn::prelude::{Backend, Device, Tensor, TensorData};
use std::iter::zip;
use std::ops::Range;

// Creates a vector of floats for writing the encoded data for |batch_size|
// board positions.
pub fn get_batch_vec(batch_size: impl Into<usize>) -> Vec<f32> {
    vec![0.0; batch_size.into() * N_PLANES * BOARD_SIZE]
}

// Returns a pair representing the first and last index in a range for a given
// |batch|.
pub fn get_batch_range(batch: impl Into<usize>) -> Range<usize> {
    let first = batch.into() * N_PLANES * BOARD_SIZE;
    let last = first + N_PLANES * BOARD_SIZE;
    first..last
}

pub trait TensorEncoder<B: Backend> {
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
#[derive(Clone, Debug)]
pub struct AzEncoder<B: Backend> {
    device: Device<B>,
}

impl<B: Backend> AzEncoder<B> {
    pub fn new(device: Device<B>) -> Self {
        Self { device }
    }
}

impl<B: Backend> TensorEncoder<B> for AzEncoder<B> {
    // Outputs a Tensor with dimensions (1, 119, 8, 8).
    fn encode_board(&self, board: &Board) -> Tensor<B, 4> {
        let mut data = vec![0.0; N_PLANES * BOARD_SIZE];
        let six_planes = N_PIECE_TYPES * BOARD_SIZE;

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
        for (val, chunk) in zip(
            state_features.iter().rev(),
            data.rchunks_exact_mut(BOARD_SIZE),
        ) {
            chunk.fill(*val);
        }

        let tensor_data = TensorData::new(data, [1, N_PLANES, N_ROWS, N_COLS]);
        Tensor::from_data(tensor_data, &self.device)
    }

    fn encode_boards(&self, boards: &[Board]) -> Tensor<B, 4> {
        assert!(!boards.is_empty());

        let mut data = vec![0.0; N_PLANES * BOARD_SIZE];
        let six_planes = N_PIECE_TYPES * BOARD_SIZE;

        for (board, chunk) in zip(
            boards.into_iter().rev().take(8),
            data.chunks_exact_mut(BOARD_SIZE * 14),
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

        for (val, chunk) in zip(
            state_features.iter().rev(),
            data.rchunks_exact_mut(BOARD_SIZE),
        ) {
            chunk.fill(*val);
        }

        let tensor_data = TensorData::new(data, [1, N_PLANES, N_ROWS, N_COLS]);
        Tensor::from_data(tensor_data, &self.device)
    }

    // Encodes the move probabilities in mps as a tensor.
    fn encode_mps(&self, mps: &[Mp]) -> Tensor<B, 4> {
        assert!(!mps.is_empty());
        let mut data = vec![0.0; N_MOVE_TYPES * BOARD_SIZE];
        let total_visits = mps.iter().fold(0, |acc, mp| acc + mp.visits) as f32;
        for mp in mps {
            let ec_move = EcMove::from(mp.pm);
            let index = ec_move.index();
            data[index] = mp.visits as f32 / total_visits;
        }
        let tensor_data = TensorData::new(data, [1, N_MOVE_TYPES, N_ROWS, N_COLS]);
        Tensor::from_data(tensor_data, &self.device)
    }

    // Encodes the move probabilities in bps as a tensor.
    fn encode_bps(&self, bps: &[Bp]) -> Tensor<B, 4> {
        assert!(!bps.is_empty());
        let mut data = vec![0.0; N_MOVE_TYPES * BOARD_SIZE];
        let total_visits = bps.iter().fold(0, |acc, bp| acc + bp.visits) as f32;
        for bp in bps {
            let ec_move = EcMove::from(bp.last_move());
            let index = ec_move.index();
            data[index] = bp.visits as f32 / total_visits;
        }
        let tensor_data = TensorData::new(data, [1, N_MOVE_TYPES, N_ROWS, N_COLS]);
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
    assert!(data.len() == N_PIECE_TYPES * BOARD_SIZE);
    for (piece, chunk) in zip(pieces.iter(), data.chunks_exact_mut(BOARD_SIZE)) {
        for sq in piece.val().sq_iter() {
            chunk[sq.as_usize()] = 1.0;
        }
    }
}

const BOARD_SIZE: usize = 64;
const N_PIECE_TYPES: usize = 6;
const N_ROWS: usize = 8;
const N_COLS: usize = 8;
const N_PLANES: usize = 119;
const N_MOVE_TYPES: usize = 6;

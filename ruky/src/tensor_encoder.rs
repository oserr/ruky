// This module contains components for encoding boards and moves to tensors.

use crate::board::Board;
use crate::piece_set::PieceSet;
use crate::search::{Bp, Mp};
use burn::prelude::{Backend, Tensor};

trait TensorEncoder<B: Backend> {
    fn encode_board(board: &Board) -> Tensor<B, 4>;
    fn encode_boards(boards: &[Board]) -> Tensor<B, 4>;
    fn encode_mps(mps: &[Mp]) -> Tensor<B, 4>;
    fn encode_bps(bps: &[Bp]) -> Tensor<B, 4>;
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
struct AzEncoder;

fn encode_pieces(pieces: &PieceSet, data: &mut [f32]) {
    assert!(data.len() == 6 * 64);
    for (piece, chunk) in std::iter::zip(pieces.iter(), data.chunks_exact_mut(64)) {
        for sq in piece.val().sq_iter() {
            chunk[sq.as_usize()] = 1.0;
        }
    }
}

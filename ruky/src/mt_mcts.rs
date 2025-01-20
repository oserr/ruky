// This module contains components for building a multi-threaded MCTS.

use crate::eval::Eval;
use std::sync::Arc;

// Represents a Multi-thread self-play MCTS.
//
// The parallel MCTS uses a leader-worker architecture, with three different
// types of workers:
// * Encoding workers that call Board::next_boards() to get the next boards from
//   a given position, and which encode the board state into a format that can
//   be directly consumed by the evaluator.
// * Decoding workers that decode the output of the evaluator, i.e. the move
//   priors and the value for the given positions.
// * Evaluation workers that take the encoded input to evaluate the positions.
//
// Note that a single thread will do both encoding and decoding, depending on
// the type of the input. The leader controls the rollouts and schedules all the
// work.
//
// Algo:
// while we have more simulations
//   do rollouts until we have BATCH_SIZE nodes to expand or we finish the
// simulations:
//     - if node needs expansion, then
//       - add node to queue for encoding
//       - do incomplete update
//     - do complete update
//   evalute nodes
//   decode nodes
//   do complete update
//
// TODO: flesh out implemention.
#[derive(Debug)]
pub struct MtSpMcts<E: Eval> {
    evaluator: Arc<E>,
    sims: u32,
    // If true, noise is added to the move priors for the root node.
    use_noise: bool,
    // If true, the MCTS samples from the moves, rather than returning the move
    // with the highest visit count.
    sample_action: bool,
    // The maximum number of boards that are sent for eval to the evaluator.
    batch_size: u8,
    // The number of workers to use for encoding and decoding board positions.
    num_workers: u8,
}

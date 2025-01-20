// This module contains components for building a multi-threaded MCTS.

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
pub struct MtSpMcts {}

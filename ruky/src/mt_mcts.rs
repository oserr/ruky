// This module contains components for building a multi-threaded MCTS.

use crate::err::RukyErr;
use crate::eval::Eval;
use crate::search::{Bp, SearchResult, SpSearch};
use crate::tensor_encoder::{get_batch_range, get_batch_vec};
use crate::tree_search::TreeSearch;
use crossbeam::channel::{Receiver, Sender};
use rayon::ThreadPool;
use std::cmp::{max, min};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

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
    tree_search: TreeSearch,
    work_pool: ThreadPool,
    // Sends encoding and decoding work to the workers.
    work_tx: Sender<Task>,
    // Receives decoded work from the workers.
    decoded_rx: Receiver<Task>,
    // The total number of simulations to run.
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

impl<E: Eval> SpSearch for MtSpMcts<E> {
    fn search(&mut self) -> Result<SearchResult, RukyErr> {
        let search_start = Instant::now();

        let root_index = self.tree_search.root_index();
        self.tree_search.sample_action = self.sample_action;

        let mut eval_time = if self.tree_search.is_root_leaf() {
            let eval_time = Instant::now();
            let eval_boards = self.evaluator.eval(self.tree_search.root_board())?;
            let eval_time = eval_time.elapsed();
            self.tree_search.expand(root_index, eval_boards);
            eval_time
        } else {
            Duration::ZERO
        };

        if self.use_noise {
            self.tree_search.add_priors_noise(root_index);
        }

        let mut max_depth = 0u32;
        let mut nodes_expanded = 0;
        let mut nodes_visited = 0;
        let mut completed_sims = 0;

        while completed_sims < self.sims {
            let mut batch_count = 0;
            let total_batch_count = min(self.sims - completed_sims, self.batch_size.into());
            // Todo: create big enough vector to be able to hold total batch count.
            let mut data = get_batch_vec(total_batch_count as usize);
            let mutex_cond = Arc::new((Mutex::new(0), Condvar::new()));

            while batch_count < total_batch_count && completed_sims < self.sims {
                let rollout = self.tree_search.rollout()?;
                let (node_id, depth) = rollout.info();

                max_depth = max(max_depth, depth);
                nodes_visited += depth;
                completed_sims += 1;

                if rollout.is_terminal() {
                    continue;
                }

                let _enc_task = EncTask {
                    node_id,
                    mutex_cond: mutex_cond.clone(),
                    data: data
                        .get_mut(get_batch_range(batch_count as usize))
                        .expect("Expecting batch of data."),
                };
                batch_count += 1;
                // TODO: Add encoding work task, and do incomplete update.
            }

            // TODO:
            // - wait until workers are done with encoding.
            // - send for eval
            // - add decoding work tasks
            // - do complete update

            let board = self.tree_search.board(root_index);
            let eval_start = Instant::now();
            let eval_boards = self.evaluator.eval(board)?;
            eval_time += eval_start.elapsed();
            self.tree_search.expand(root_index, eval_boards);
            nodes_expanded += 1;
        }

        let best_node = self.tree_search.select_action();
        let result = SearchResult {
            best: Bp::from(best_node),
            moves: self.tree_search.move_probs(),
            value: best_node.value,
            nodes_expanded,
            nodes_visited,
            depth: max_depth,
            total_eval_time: eval_time,
            total_search_time: search_start.elapsed(),
        };
        self.tree_search.update_root_from_index(best_node.index);
        Ok(result)
    }
}

// An enum to represent the different types of work.
#[derive(Debug)]
pub enum Task {
    Decode,
    Encode,
}

#[derive(Debug)]
struct EncTask<'a> {
    node_id: usize,
    mutex_cond: Arc<(Mutex<u32>, Condvar)>,
    data: &'a mut [f32],
}

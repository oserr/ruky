// This module contains components for building a multi-threaded MCTS.

use crate::err::RukyErr;
use crate::eval::Eval;
use crate::search::{Bp, SearchResult, SpSearch};
use crate::tensor_decoder::N_POSSIBLE_MOVES;
use crate::tensor_encoder::{enc_board, get_batch_vec, single_batch_size};
use crate::tree_search::TreeSearch;
use crate::Board;
use crossbeam::channel::{unbounded, Receiver, Sender};
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::cmp::{max, min};
use std::iter::zip;
use std::sync::Arc;
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
    // Receives encoded work from the workers.
    encoded_rx: Receiver<EncResult>,
    // Receives decoded work from the workers.
    decoded_rx: Receiver<DecResult>,
    // The total number of simulations to run.
    sims: u32,
    // If true, noise is added to the move priors for the root node.
    use_noise: bool,
    // If true, the MCTS samples from the moves, rather than returning the move
    // with the highest visit count.
    sample_action: bool,
    // The maximum number of boards that are sent for eval to the evaluator.
    batch_size: u32,
    // The number of workers to use for encoding and decoding board positions.
    num_workers: u32,
}

impl<E: Eval> MtSpMcts<E> {
    // Initialiazes the MCTS by creating a tool of worker threads to parallelize
    // encoding and decoding tasks.
    pub fn create(
        evaluator: Arc<E>,
        tree_search: TreeSearch,
        sims: u32,
        use_noise: bool,
        sample_action: bool,
        batch_size: u32,
        num_workers: u32,
    ) -> Self {
        let work_pool = ThreadPoolBuilder::new()
            .num_threads(num_workers as usize)
            .build()
            .expect("Expecting thread pool.");

        let (work_tx, work_rx) = unbounded();
        let (encoded_tx, encoded_rx) = unbounded();
        let (decoded_tx, decoded_rx) = unbounded();

        for _ in 0..num_workers {
            let work_rx = work_rx.clone();
            let decoded_tx = decoded_tx.clone();
            let encoded_tx = encoded_tx.clone();
            work_pool.spawn(move || {
                for work in work_rx {
                    match work {
                        Task::Encode(enc_task) => {
                            let result = enc_task.run_task();
                            if let Err(_) = encoded_tx.send(result) {
                                break;
                            }
                        }
                        Task::Decode(dec_task) => {
                            let result = dec_task.run_task();
                            if let Err(_) = decoded_tx.send(result) {
                                break;
                            }
                        }
                    }
                }
            });
        }

        Self {
            evaluator,
            tree_search,
            work_pool,
            work_tx,
            encoded_rx,
            decoded_rx,
            sims,
            use_noise,
            sample_action,
            batch_size,
            num_workers,
        }
    }
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

            while batch_count < total_batch_count && completed_sims < self.sims {
                let rollout = self.tree_search.rollout()?;
                let (node_id, depth) = rollout.info();

                max_depth = max(max_depth, depth);
                nodes_visited += depth;
                completed_sims += 1;

                if rollout.is_terminal() {
                    continue;
                }

                let enc_task = EncTask {
                    node_id,
                    board: self.tree_search.board(node_id).clone(),
                };
                self.work_tx
                    .send(Task::Encode(enc_task))
                    .expect("Encoding task should be transmitted.");
                self.tree_search.incomplete_update(node_id);
                batch_count += 1;
            }

            let mut data = get_batch_vec(batch_count as usize);
            let enc_results = self
                .encoded_rx
                .iter()
                .take(batch_count as usize)
                .collect::<Vec<_>>();

            assert_eq!(enc_results.len(), batch_count as usize);

            for (data_batch, enc_result) in data
                .chunks_exact_mut(single_batch_size())
                .zip(enc_results.iter())
            {
                data_batch.copy_from_slice(enc_result.enc_data.as_ref());
            }

            let eval_start = Instant::now();
            let (mv_data, value_data) =
                self.evaluator.eval_batch_data(batch_count as usize, data)?;
            eval_time += eval_start.elapsed();

            for ((enc_moves, value), enc_result) in zip(
                mv_data.chunks_exact(N_POSSIBLE_MOVES).zip(value_data),
                enc_results,
            ) {
                let dec_task = DecTask {
                    node_id: enc_result.node_id,
                    moves: enc_result.moves,
                    enc_moves: enc_moves.to_vec(),
                    value,
                };
                self.work_tx
                    .send(Task::Decode(dec_task))
                    .expect("Decoding task should be transmitted.");
            }

            // TODO:
            // - do complete update

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
enum Task {
    Decode(DecTask),
    Encode(EncTask),
}

// A struct representing a decoding task.
#[derive(Clone, Debug)]
struct DecTask {
    node_id: usize,
    moves: Vec<Board>,
    enc_moves: Vec<f32>,
    value: f32,
}

// A struct representing a decoded result.
struct DecResult {}

impl DecTask {
    fn run_task(&self) -> DecResult {
        todo!();
    }
}

// A struct representing an encoding task.
#[derive(Debug)]
struct EncTask {
    node_id: usize,
    board: Board,
}

// A struct representing an encoded result.
#[derive(Clone, Debug)]
struct EncResult {
    node_id: usize,
    board: Board,
    moves: Vec<Board>,
    enc_data: Vec<f32>,
}

impl EncTask {
    fn run_task(self) -> EncResult {
        let moves = self
            .board
            .next_boards()
            .expect("Expecting moves from non-terminal board.");
        let enc_data = enc_board(&self.board);

        EncResult {
            node_id: self.node_id,
            board: self.board,
            moves,
            enc_data,
        }
    }
}

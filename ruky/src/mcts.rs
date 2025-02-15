// This module contains the components for the MCTS search.

use crate::board::Board;
use crate::err::RukyErr;
use crate::eval::Eval;
use crate::search::{Bp, Search, SearchResult, SpSearch, TreeSize};
use crate::tree_search::TreeSearch;
use std::cmp::max;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct SpMcts<E: Eval> {
    evaluator: Arc<E>,
    search_tree: TreeSearch,
    sims: u32,
    use_noise: bool,
    sample_action: bool,
}

impl<E: Eval> TreeSize for SpMcts<E> {
    fn total_tree_nodes(&self) -> usize {
        self.search_tree.total_tree_nodes()
    }
}

impl<E: Eval> SpSearch for SpMcts<E> {
    fn search(&mut self) -> Result<SearchResult, RukyErr> {
        let search_start = Instant::now();

        let root_index = self.search_tree.root_index();
        self.search_tree.sample_action = self.sample_action;

        let mut eval_time = if self.search_tree.is_root_leaf() {
            let eval_time = Instant::now();
            let eval_boards = self.evaluator.eval(self.search_tree.root_board())?;
            let eval_time = eval_time.elapsed();
            self.search_tree.expand(root_index, eval_boards);
            eval_time
        } else {
            Duration::ZERO
        };

        if self.use_noise {
            self.search_tree.add_priors_noise(root_index);
        }

        let mut max_depth = 0u32;
        let mut nodes_expanded = 1;
        let mut nodes_visited = 0;

        for _ in 0..self.sims {
            let mut node_index = root_index;
            let mut current_depth = 0u32;
            while self.search_tree.is_expanded(node_index) {
                current_depth += 1;
                nodes_visited += 1;
                node_index = self
                    .search_tree
                    .choose_next(node_index)
                    .ok_or(RukyErr::SearchChooseNext)?;
            }
            max_depth = max(max_depth, current_depth);
            if self.search_tree.is_terminal(node_index) {
                self.search_tree.terminate(node_index);
                continue;
            }
            let board = self.search_tree.board(node_index);
            let eval_start = Instant::now();
            let eval_boards = self.evaluator.eval(board)?;
            eval_time += eval_start.elapsed();
            self.search_tree.expand(node_index, eval_boards);
            nodes_expanded += 1
        }

        let best_node = self.search_tree.select_action();
        let result = SearchResult {
            best: Bp::from(best_node),
            moves: self.search_tree.move_probs(),
            value: best_node.value,
            nodes_expanded,
            nodes_visited,
            depth: max_depth,
            total_evals: 0,
            total_eval_time: eval_time,
            total_search_time: search_start.elapsed(),
            avg_move_gen_time: Duration::ZERO,
            max_move_gen_time: Duration::ZERO,
        };
        self.search_tree.update_root_from_index(best_node.index);
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct SpMctsBuilder<E: Eval> {
    eval: Option<Arc<E>>,
    board: Option<Board>,
    sims: u32,
    use_noise: bool,
    sample_action: bool,
}

impl<E: Eval> SpMctsBuilder<E> {
    pub fn new() -> Self {
        Self {
            eval: None,
            board: None,
            sims: 800,
            use_noise: true,
            sample_action: true,
        }
    }

    pub fn eval(mut self, evaluator: Arc<E>) -> Self {
        self.eval.replace(evaluator);
        self
    }

    pub fn board(mut self, board: Board) -> Self {
        self.board.replace(board);
        self
    }

    pub fn sims(mut self, sims: u32) -> Self {
        self.sims = sims;
        self
    }

    pub fn use_noise(mut self, use_noise: bool) -> Self {
        self.use_noise = use_noise;
        self
    }

    pub fn sample_action(mut self, sample_action: bool) -> Self {
        self.sample_action = sample_action;
        self
    }

    pub fn build(self) -> Result<SpMcts<E>, RukyErr> {
        match (self.eval, self.board) {
            (Some(eval), Some(board)) => Ok(SpMcts {
                evaluator: eval,
                search_tree: TreeSearch::with_capacity(board, 6_000_000),
                sims: self.sims,
                use_noise: self.use_noise,
                sample_action: self.sample_action,
            }),
            _ => Err(RukyErr::PreconditionErr),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mcts<E: Eval> {
    evaluator: Arc<E>,
    search_tree: TreeSearch,
    sims: u32,
    use_noise: bool,
    sample_action: bool,
}

impl<E: Eval> Mcts<E> {
    pub fn create(evaluator: Arc<E>, sims: u32) -> Self {
        Self {
            evaluator,
            search_tree: TreeSearch::new(),
            sims,
            use_noise: false,
            sample_action: false,
        }
    }

    pub fn create_with_noise(evaluator: Arc<E>, sims: u32) -> Self {
        Self {
            evaluator,
            search_tree: TreeSearch::new(),
            sims,
            use_noise: true,
            sample_action: false,
        }
    }

    pub fn enable_sample_action(&mut self, sample_action: bool) {
        self.sample_action = sample_action;
    }
}

impl<E: Eval> Search for Mcts<E> {
    fn search_board(&mut self, board: &Board) -> Result<SearchResult, RukyErr> {
        let boards = [board.clone()];
        self.search_game(boards.as_ref())
    }

    fn search_game(&mut self, boards: &[Board]) -> Result<SearchResult, RukyErr> {
        let board = boards.last().ok_or(RukyErr::SearchMissingBoard)?;
        if board.is_terminal() {
            return Err(RukyErr::SearchTerminalBoard);
        }
        let search_start = Instant::now();

        self.search_tree.update_root_from_board(board);
        let root_index = self.search_tree.root_index();
        self.search_tree.sample_action = self.sample_action;

        let mut eval_time = if self.search_tree.is_root_leaf() {
            let eval_time = Instant::now();
            let eval_boards = self.evaluator.eval(board)?;
            let eval_time = eval_time.elapsed();
            self.search_tree.expand(root_index, eval_boards);
            eval_time
        } else {
            Duration::ZERO
        };

        if self.use_noise {
            self.search_tree.add_priors_noise(root_index);
        }

        let mut max_depth = 0u32;
        let mut nodes_expanded = 1;
        let mut nodes_visited = 0;

        for _ in 0..self.sims {
            let mut node_index = root_index;
            let mut current_depth = 0u32;
            while self.search_tree.is_expanded(node_index) {
                current_depth += 1;
                nodes_visited += 1;
                node_index = self
                    .search_tree
                    .choose_next(node_index)
                    .ok_or(RukyErr::SearchChooseNext)?;
            }
            max_depth = max(max_depth, current_depth);
            if self.search_tree.is_terminal(node_index) {
                self.search_tree.terminate(node_index);
                continue;
            }
            let board = self.search_tree.board(node_index);
            let eval_start = Instant::now();
            let eval_boards = self.evaluator.eval(board)?;
            eval_time += eval_start.elapsed();
            self.search_tree.expand(node_index, eval_boards);
            nodes_expanded += 1
        }

        let best_node = self.search_tree.select_action();
        let result = SearchResult {
            best: Bp::from(best_node),
            moves: self.search_tree.move_probs(),
            value: best_node.value,
            nodes_expanded,
            nodes_visited,
            depth: max_depth,
            total_evals: 0,
            total_eval_time: eval_time,
            total_search_time: search_start.elapsed(),
            avg_move_gen_time: Duration::ZERO,
            max_move_gen_time: Duration::ZERO,
        };
        self.search_tree.update_root_from_index(best_node.index);
        Ok(result)
    }
}

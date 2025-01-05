// This module contains the components for the MCTS search.

use crate::board::Board;
use crate::err::RukyErr;
use crate::eval::{Eval, EvalBoards};
use crate::search::{Bp, Mp, Search, SearchResult, SpSearch};
use rand::{distributions::weighted::WeightedIndex, thread_rng};
use rand_distr::{Dirichlet, Distribution};
use std::cmp::max;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct SpMcts<E: Eval> {
    evaluator: Arc<E>,
    search_tree: SearchTree,
    sims: u32,
    use_noise: bool,
    sample_action: bool,
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
            total_eval_time: eval_time,
            total_search_time: search_start.elapsed(),
        };
        self.search_tree.update_root_from_index(best_node.index);
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct Mcts<E: Eval> {
    evaluator: Arc<E>,
    search_tree: SearchTree,
    sims: u32,
    use_noise: bool,
    sample_action: bool,
}

impl<E: Eval> Mcts<E> {
    pub fn create(evaluator: Arc<E>, sims: u32) -> Self {
        Self {
            evaluator,
            search_tree: SearchTree::new(),
            sims,
            use_noise: false,
            sample_action: false,
        }
    }

    pub fn create_with_noise(evaluator: Arc<E>, sims: u32) -> Self {
        Self {
            evaluator,
            search_tree: SearchTree::new(),
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
            total_eval_time: eval_time,
            total_search_time: search_start.elapsed(),
        };
        self.search_tree.update_root_from_index(best_node.index);
        Ok(result)
    }
}

#[derive(Clone, Debug)]
struct SearchTree {
    children: Vec<Node>,
    root: usize,
    sample_action: bool,
}

impl Default for SearchTree {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            root: 0,
            sample_action: false,
        }
    }
}

impl SearchTree {
    fn new() -> Self {
        SearchTree::default()
    }

    fn choose_next(&self, parent_index: usize) -> Option<usize> {
        let parent_node = &self.children[parent_index];
        assert!(!parent_node.is_leaf);
        let child_visits = self.child_visits(parent_index);
        self.children[parent_node.children.0..parent_node.children.1]
            .iter()
            .reduce(|acc_node, node| {
                let acc_node_score = acc_node.score(parent_node.visits, child_visits);
                let node_score = node.score(parent_node.visits, child_visits);
                if acc_node_score > node_score {
                    acc_node
                } else {
                    node
                }
            })
            .map(|node| node.index)
    }

    fn child_visits(&self, parent_index: usize) -> u32 {
        let parent_node = &self.children[parent_index];
        self.children[parent_node.children.0..parent_node.children.1]
            .iter()
            .fold(0, |visits, node| visits + node.visits)
    }

    fn board(&self, node_index: usize) -> &Board {
        &self.children[node_index].board
    }

    fn is_leaf(&self, node_index: usize) -> bool {
        self.children[node_index].is_leaf
    }

    fn is_terminal(&self, node_index: usize) -> bool {
        self.children[node_index].is_terminal()
    }

    fn is_expanded(&self, node_index: usize) -> bool {
        !self.is_leaf(node_index) && !self.is_terminal(node_index)
    }

    fn root_index(&self) -> usize {
        self.root
    }

    fn root_node(&self) -> &Node {
        &self.children[self.root]
    }

    fn root_board(&self) -> &Board {
        &self.root_node().board
    }

    fn is_root_leaf(&self) -> bool {
        self.children[self.root].is_leaf
    }

    fn terminate(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        assert!(node.is_terminal());
        node.init_value = match node.board.is_mate() {
            true => 1.0,
            false => 0.0,
        };
        node.value = node.init_value;
        self.update_nodes(node_index);
    }

    fn update_nodes(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        node.visits += 1;
        let mut val = node.value;
        let mut parent = node.parent;
        while parent.is_some() {
            let node = &mut self.children[parent.unwrap()];
            val *= -1.0;
            node.visits += 1;
            node.value += val;
            parent = node.parent;
        }
    }

    fn expand(&mut self, node_index: usize, eval_boards: EvalBoards) {
        let first_index = self.children.len();
        let last_index = first_index + eval_boards.board_probs.len();
        let node = &mut self.children[node_index];
        node.children = (first_index, last_index);
        node.init_value = eval_boards.value;
        node.value = node.init_value;
        node.is_leaf = false;
        self.children
            .extend(eval_boards.board_probs.into_iter().zip(first_index..).map(
                |((board, prior), index)| {
                    Node::from_board_parent_prior_index(board, node_index, prior, index)
                },
            ));
        self.update_nodes(node_index);
    }

    fn most_visited(&self) -> &Node {
        let (first, last) = self.children[self.root].children;
        self.children[first..last]
            .iter()
            .max_by_key(|node| node.visits)
            .expect("Expecting at least one move in non-terminal state.")
    }

    fn sample_most_visited(&self) -> &Node {
        let (first, last) = self.children[self.root].children;
        let weights: Vec<_> = self.children[first..last]
            .iter()
            .map(|node| node.visits)
            .collect();
        let weighted_dist = WeightedIndex::new(&weights).expect("Expecting WeightedIndex.");
        let index = weighted_dist.sample(&mut thread_rng());
        &self.children[first + index]
    }

    fn select_action(&self) -> &Node {
        let num_actions = self.num_actions();
        assert!(num_actions >= 1);
        match self.sample_action {
            true if num_actions > 1 => self.sample_most_visited(),
            _ => self.most_visited(),
        }
    }

    fn move_probs(&self) -> Vec<Mp> {
        let (first, last) = self.children[self.root].children;
        self.children[first..last]
            .iter()
            .map(|node| Mp::from(node))
            .collect()
    }

    fn num_actions(&self) -> usize {
        let (first, last) = self.children[self.root].children;
        last - first
    }

    fn update_root_from_board(&mut self, board: &Board) {
        if self.children.is_empty() {
            self.children.push(Node::from(board));
            return;
        }
        let current_root = &self.children[self.root];
        assert!(!current_root.is_leaf);
        let (first, last) = current_root.children;
        match self.children[first..last]
            .iter()
            .find(|node| node.board == *board)
        {
            Some(ref node) => {
                self.root = node.index;
            }
            None => {
                self.children.clear();
                self.children.push(Node::from(board));
                self.root = 0;
            }
        };
    }

    fn update_root_from_index(&mut self, new_root: usize) {
        self.root = new_root;
    }

    fn add_priors_noise(&mut self, node_index: usize) {
        let (first, last) = self.children[node_index].children;
        let n_moves = last - first;
        if n_moves < 2 {
            return;
        }
        let dirichlet = Dirichlet::new_with_size(DIR_ALPHA, n_moves)
            .expect("Expecting Dirichlet distribution.");
        let probs = dirichlet.sample(&mut thread_rng());
        for (node, noise) in self.children[first..last].iter_mut().zip(probs) {
            node.prior = (1.0 - DIR_EXPLORE_FRAC) * node.prior + DIR_EXPLORE_FRAC * noise;
        }
    }
}

impl From<&Board> for SearchTree {
    fn from(board: &Board) -> Self {
        Self {
            children: vec![Node::from(board)],
            root: 0,
            sample_action: false,
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    // Represents the current board position.
    board: Board,
    // Represenst the first and last index in the array where the children are located.
    children: (usize, usize),
    // If this is a child node, then parent points to the index where the parent is located.
    parent: Option<usize>,
    // The index where this node is located.
    index: usize,
    // The prior probability of this node.
    prior: f32,
    // The number of times this node is visited during search.
    visits: u32,
    // The total value of the node. This includes all possible variations explored from this node.
    value: f32,
    // The value of this position, as computed by the evaluator or [0, 1, -1] if this is known to
    // be a draw, win, or loss from the perspective of the player who made the move leading up to
    // this position.
    init_value: f32,
    // True if this node has not been expanded yet, false otherwise.
    is_leaf: bool,
}

impl From<&Node> for Bp {
    fn from(node: &Node) -> Self {
        Self {
            board: node.board.clone(),
            prior: node.prior,
            visits: node.visits,
        }
    }
}

impl From<&Node> for Mp {
    fn from(node: &Node) -> Self {
        Self {
            pm: node
                .board
                .last_move()
                .expect("A move should have led to this node."),
            prior: node.prior,
            visits: node.visits,
        }
    }
}

// Creates a Node from a Board by taking ownership of the board.
impl From<Board> for Node {
    fn from(board: Board) -> Self {
        Self {
            board: board,
            children: (0, 0),
            parent: None,
            index: 0,
            prior: 0.0,
            visits: 0,
            value: 0.0,
            init_value: 0.0,
            is_leaf: true,
        }
    }
}

// Creates a Node from a reference to a Board.
impl From<&Board> for Node {
    fn from(board: &Board) -> Self {
        Node::from(board.clone())
    }
}

impl Node {
    // Creates a Node from a board and a prior, but takes ownership of the board.
    fn from_board_parent_prior_index(
        board: Board,
        parent: usize,
        prior: f32,
        index: usize,
    ) -> Self {
        let mut node = Node::from(board);
        node.parent = Some(parent);
        node.prior = prior;
        node.index = index;
        node
    }

    fn score(&self, parent_visits: u32, sibling_visits: u32) -> f32 {
        match self.visits {
            0 => self.ucb(parent_visits, sibling_visits),
            _ => self.value / self.visits as f32 + self.ucb(parent_visits, sibling_visits),
        }
    }

    fn ucb(&self, parent_visits: u32, sibling_visits: u32) -> f32 {
        let term1 = explore_rate(parent_visits) * self.prior;
        let term2 = (sibling_visits as f32).sqrt() / (1 + self.visits) as f32;
        term1 * term2
    }

    fn is_terminal(&self) -> bool {
        self.board.is_terminal()
    }
}

fn explore_rate(parent_visits: u32) -> f32 {
    let num = 1.0 + parent_visits as f32 + EXPLORE_BASE;
    (num / EXPLORE_BASE).ln() + EXPLORE_INIT
}

const EXPLORE_BASE: f32 = 19652.0;
const EXPLORE_INIT: f32 = 1.25;
const DIR_ALPHA: f32 = 0.3;
const DIR_EXPLORE_FRAC: f32 = 0.25;

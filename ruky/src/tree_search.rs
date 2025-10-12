// This module contains a TreeSearch component.

use crate::err::RukyErr;
use crate::eval::EvalBoards;
use crate::search::{Bp, Mp, TreeSize};
use crate::Board;
use rand::{distr::weighted::WeightedIndex, rng};
use rand_distr::{Distribution, Gamma};

#[derive(Clone, Debug)]
pub struct TreeSearch {
    children: Vec<Node>,
    root: usize,
    pub sample_action: bool,
}

impl Default for TreeSearch {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            root: 0,
            sample_action: false,
        }
    }
}

impl TreeSize for TreeSearch {
    fn total_tree_nodes(&self) -> usize {
        self.children.len()
    }
}

impl TreeSearch {
    pub fn new() -> Self {
        TreeSearch::default()
    }

    pub fn with_capacity(board: Board, capacity: usize) -> Self {
        let mut children = Vec::with_capacity(capacity);
        children.push(Node::from(board));
        Self {
            children,
            root: 0,
            sample_action: false,
        }
    }

    pub fn reset(&mut self) {
        let node = Node::from(self.board(0).clone());
        self.children.clear();
        self.children.push(node);
        self.root = 0;
        self.sample_action = false;
    }

    pub fn rollout(&mut self) -> Result<RolloutType, RukyErr> {
        let mut node_index = self.root_index();
        let mut depth = 0u32;
        while self.is_expanded(node_index) {
            depth += 1;
            node_index = self
                .choose_next(node_index)
                .ok_or(RukyErr::SearchChooseNext)?;
        }
        if self.is_terminal(node_index) {
            self.terminate(node_index);
            Ok(RolloutType::Terminal {
                node_id: node_index,
                depth,
            })
        } else {
            Ok(RolloutType::Leaf {
                node_id: node_index,
                depth,
            })
        }
    }

    pub fn choose_next(&self, parent_index: usize) -> Option<usize> {
        let parent_node = &self.children[parent_index];
        assert!(!parent_node.is_leaf);
        let child_visits = self.child_visits(parent_index);
        self.children[parent_node.children.0..parent_node.children.1]
            .iter()
            .reduce(|acc_node, node| {
                let parent_visits = parent_node.total_visits();
                let acc_node_score = acc_node.score(parent_visits, child_visits);
                let node_score = node.score(parent_visits, child_visits);
                if acc_node_score > node_score {
                    acc_node
                } else {
                    node
                }
            })
            .map(|node| node.index)
    }

    pub fn child_visits(&self, parent_index: usize) -> u32 {
        let parent_node = &self.children[parent_index];
        self.children[parent_node.children.0..parent_node.children.1]
            .iter()
            .fold(0, |visits, node| visits + node.total_visits())
    }

    pub fn board(&self, node_index: usize) -> &Board {
        &self.children[node_index].board
    }

    pub fn is_leaf(&self, node_index: usize) -> bool {
        self.children[node_index].is_leaf
    }

    pub fn is_terminal(&self, node_index: usize) -> bool {
        self.children[node_index].is_terminal()
    }

    pub fn is_expanded(&self, node_index: usize) -> bool {
        !self.is_leaf(node_index) && !self.is_terminal(node_index)
    }

    pub fn root_index(&self) -> usize {
        self.root
    }

    pub fn root_node(&self) -> &Node {
        &self.children[self.root]
    }

    pub fn root_board(&self) -> &Board {
        &self.root_node().board
    }

    pub fn is_root_leaf(&self) -> bool {
        self.children[self.root].is_leaf
    }

    pub fn terminate(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        assert!(node.is_terminal());
        node.init_value = match node.board.is_mate() {
            true => 1.0,
            false => 0.0,
        };
        node.value = node.init_value;
        self.update_nodes(node_index);
    }

    pub fn update_nodes(&mut self, node_index: usize) {
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

    pub fn incomplete_update(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        node.partial_visits += 1;
        let mut parent = node.parent;
        while parent.is_some() {
            let node = &mut self.children[parent.unwrap()];
            node.partial_visits += 1;
            parent = node.parent;
        }
    }

    pub fn complete_update(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        node.partial_visits -= 1;
        node.visits += 1;
        let mut val = node.value;
        let mut parent = node.parent;
        while parent.is_some() {
            let node = &mut self.children[parent.unwrap()];
            val *= -1.0;
            node.partial_visits -= 1;
            node.visits += 1;
            node.value += val;
            parent = node.parent;
        }
    }

    // Similar to complete_upate, but it only updates the visit counts.
    pub fn complete_visits(&mut self, node_index: usize) {
        let node = &mut self.children[node_index];
        node.partial_visits -= 1;
        node.visits += 1;
        let mut parent = node.parent;
        while parent.is_some() {
            let node = &mut self.children[parent.unwrap()];
            node.partial_visits -= 1;
            node.visits += 1;
            parent = node.parent;
        }
    }

    pub fn complete_expand(&mut self, node_index: usize, eval_boards: EvalBoards) {
        match self.only_expand(node_index, eval_boards) {
            true => self.complete_update(node_index),
            false => self.complete_visits(node_index),
        }
    }

    pub fn expand(&mut self, node_index: usize, eval_boards: EvalBoards) {
        self.only_expand(node_index, eval_boards);
        self.update_nodes(node_index);
    }

    fn only_expand(&mut self, node_index: usize, eval_boards: EvalBoards) -> bool {
        let first_index = self.children.len();
        let last_index = first_index + eval_boards.board_probs.len();
        let node = &mut self.children[node_index];
        // The node has already been expanded, hence no need to re-expand.
        if !node.is_leaf {
            return false;
        }
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
        true
    }

    pub fn most_visited(&self) -> &Node {
        let (first, last) = self.children[self.root].children;
        self.children[first..last]
            .iter()
            .max_by_key(|node| node.visits)
            .expect("Expecting at least one move in non-terminal state.")
    }

    pub fn sample_most_visited(&self) -> &Node {
        let (first, last) = self.children[self.root].children;
        let weights: Vec<_> = self.children[first..last]
            .iter()
            .map(|node| node.visits)
            .collect();
        let weighted_dist = WeightedIndex::new(&weights).expect("Expecting WeightedIndex.");
        let index = weighted_dist.sample(&mut rng());
        &self.children[first + index]
    }

    pub fn select_action(&self) -> &Node {
        let num_actions = self.num_actions();
        assert!(num_actions >= 1);
        match self.sample_action {
            true if num_actions > 1 => self.sample_most_visited(),
            _ => self.most_visited(),
        }
    }

    pub fn move_probs(&self) -> Vec<Mp> {
        let (first, last) = self.children[self.root].children;
        self.children[first..last]
            .iter()
            .map(|node| Mp::from(node))
            .collect()
    }

    pub fn num_actions(&self) -> usize {
        let (first, last) = self.children[self.root].children;
        last - first
    }

    pub fn update_root_from_board(&mut self, board: &Board) {
        if self.children.is_empty() {
            self.children.push(Node::from(board));
            return;
        }

        let current_root = &self.children[self.root];
        if current_root.board.state_hash() == board.state_hash() {
            // Don't do anything if root is the intended board.
            return;
        } else if current_root.is_leaf {
            self.children.clear();
            self.children.push(Node::from(board));
            self.root = 0;
            return;
        }

        let (first, last) = current_root.children;
        match self.children[first..last]
            .iter()
            .find(|node| node.board.state_hash() == board.state_hash())
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

    pub fn update_root_from_index(&mut self, new_root: usize) {
        self.root = new_root;
    }

    pub fn add_priors_noise(&mut self, node_index: usize) {
        let (first, last) = self.children[node_index].children;
        let n_moves = last - first;
        if n_moves < 2 {
            return;
        }
        let gamma = Gamma::new(DIR_ALPHA, 1.0).expect("Expecting Dirichlet distribution.");
        for node in self.children[first..last].iter_mut() {
            let noise = gamma.sample(&mut rng());
            node.prior = (1.0 - DIR_EXPLORE_FRAC) * node.prior + DIR_EXPLORE_FRAC * noise;
        }
    }

    // Collects the last |num_boards| leading up to board at |node_index|, starting
    // with the board at |node_index|.
    pub fn collect_last_boards(&self, node_index: usize) -> Vec<Board> {
        let mut boards: Vec<Board> = Vec::new();
        let node = &self.children[node_index];
        boards.push(node.board.clone());
        let mut parent = node.parent;
        while boards.len() < MAX_ENC_BOARDS && parent.is_some() {
            let node = &self.children[parent.unwrap()];
            boards.push(node.board.clone());
            parent = node.parent;
        }
        boards
    }
}

impl From<Board> for TreeSearch {
    fn from(board: Board) -> Self {
        Self {
            children: vec![Node::from(board)],
            root: 0,
            sample_action: false,
        }
    }
}

impl From<&Board> for TreeSearch {
    fn from(board: &Board) -> Self {
        TreeSearch::from(board.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    // Represents the current board position.
    pub board: Board,
    // Represenst the first and last index in the array where the children are located.
    pub children: (usize, usize),
    // If this is a child node, then parent points to the index where the parent is located.
    pub parent: Option<usize>,
    // The index where this node is located.
    pub index: usize,
    // The prior probability of this node.
    pub prior: f32,
    // The number of times this node is visited during search.
    pub visits: u32,
    // The number of incomplete visits this node has received.
    pub partial_visits: u32,
    // The total value of the node. This includes all possible variations explored from this node.
    pub value: f32,
    // The value of this position, as computed by the evaluator or [0, 1, -1] if this is known to
    // be a draw, win, or loss from the perspective of the player who made the move leading up to
    // this position.
    pub init_value: f32,
    // True if this node has not been expanded yet, false otherwise.
    pub is_leaf: bool,
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
            partial_visits: 0,
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
    pub fn from_board_parent_prior_index(
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

    pub fn score(&self, parent_visits: u32, sibling_visits: u32) -> f32 {
        match self.visits {
            0 => self.ucb(parent_visits, sibling_visits),
            _ => self.value / self.visits as f32 + self.ucb(parent_visits, sibling_visits),
        }
    }

    pub fn ucb(&self, parent_visits: u32, sibling_visits: u32) -> f32 {
        let term1 = explore_rate(parent_visits) * self.prior;
        let term2 = (sibling_visits as f32).sqrt() / (1 + self.total_visits()) as f32;
        term1 * term2
    }

    pub fn is_terminal(&self) -> bool {
        self.board.is_terminal()
    }

    pub fn total_visits(&self) -> u32 {
        self.visits + self.partial_visits
    }
}

// An enum to represent the result of a rollout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RolloutType {
    Terminal { node_id: usize, depth: u32 },
    Leaf { node_id: usize, depth: u32 },
}

impl RolloutType {
    pub fn is_terminal(&self) -> bool {
        matches!(self, RolloutType::Terminal { .. })
    }

    pub fn info(&self) -> (usize, u32) {
        match self {
            RolloutType::Terminal { node_id, depth } | RolloutType::Leaf { node_id, depth } => {
                (*node_id, *depth)
            }
        }
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

// The maximum number of boards to collect for encoding.
const MAX_ENC_BOARDS: usize = 8;

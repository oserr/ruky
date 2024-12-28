// This module contains the components for the MCTS search.

use crate::board::Board;
use crate::err::RukyErr;
use crate::eval::{Eval, EvalBoards};
use crate::search::{Search, SearchResult};
use std::sync::Arc;

pub struct Mcts<E: Eval> {
    evaluator: Arc<E>,
    nsim: u32,
}

impl<E: Eval> Search for Mcts<E> {
    fn search_board(&self, board: &Board) -> Result<SearchResult, RukyErr> {
        let boards = [board.clone()];
        self.search_game(boards.as_ref())
    }

    fn search_game(&self, boards: &[Board]) -> Result<SearchResult, RukyErr> {
        let board = boards.last().ok_or(RukyErr::SearchMissingBoard)?;
        let mut search_tree = SearchTree::from(board);
        for _ in 0..self.nsim {
            let mut node_index = 0;
            while !search_tree.is_leaf_or_terminal(node_index) {
                node_index = search_tree
                    .choose_next(node_index)
                    .ok_or(RukyErr::SearchChooseNext)?;
            }
            if search_tree.is_terminal(node_index) {
                search_tree.terminate(node_index);
                continue;
            }
            let board = search_tree.board(node_index);
            let eval_boards = self.evaluator.eval(board)?;
            search_tree.expand(node_index, eval_boards);
        }
        let node = search_tree.most_visited();
        // TODO: Fill in the search results.
        Ok(SearchResult::with_best(node.board.clone()))
    }
}

#[derive(Debug)]
struct SearchTree {
    children: Vec<Node>,
}

impl SearchTree {
    fn choose_next(&self, parent_index: usize) -> Option<usize> {
        let parent_node = &self.children[parent_index];
        assert!(!parent_node.is_leaf);
        self.children[parent_node.children.0..parent_node.children.1]
            .iter()
            .reduce(|acc_node, node| {
                let acc_node_uct = acc_node.mean_uct(parent_node.visits);
                let node_uct = node.mean_uct(parent_node.visits);
                if acc_node_uct > node_uct {
                    acc_node
                } else {
                    node
                }
            })
            .map(|node| node.index)
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

    fn is_leaf_or_terminal(&self, node_index: usize) -> bool {
        self.is_leaf(node_index) || self.is_terminal(node_index)
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
        self.children
            .extend(eval_boards.board_probs.into_iter().zip(first_index..).map(
                |((board, prior), index)| {
                    Node::from_board_parent_prior_index(board, node_index, prior, index)
                },
            ));
        self.update_nodes(node_index);
    }

    fn most_visited(&self) -> &Node {
        let (first, last) = self.children[0].children;
        self.children[first..last]
            .iter()
            .max_by_key(|node| node.visits)
            .expect("Expecting at least one move in non-terminal state.")
    }
}

impl From<&Board> for SearchTree {
    fn from(board: &Board) -> Self {
        Self {
            children: vec![Node::from(board)],
        }
    }
}

#[derive(Debug)]
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
    // Creates a Node from a board and a prior.
    fn with_prior(board: &Board, prior: f32) -> Self {
        let mut node = Node::from(board);
        node.prior = prior;
        node
    }

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

    // Creates a Node from a board, a parent, and a prior.
    fn with_parent_and_prior(board: &Board, parent: usize, prior: f32) -> Node {
        let mut node = Node::from(board);
        node.parent = Some(parent);
        node.prior = prior;
        node
    }

    fn mean_uct(&self, parent_visits: u32) -> f32 {
        self.value / self.visits as f32 + self.uct(parent_visits)
    }

    fn uct(&self, parent_visits: u32) -> f32 {
        let term1 = explore_rate(parent_visits) * self.prior;
        let term2 = (parent_visits as f32).sqrt() / (1 + self.visits) as f32;
        term1 * term2
    }

    fn is_terminal(&self) -> bool {
        self.board.is_terminal()
    }
}

fn explore_rate(parent_visits: u32) -> f32 {
    let num = 1.0 + parent_visits as f32 + EXPLORE_BASE;
    (num / EXPLORE_BASE).log2() + EXPLORE_INIT
}

const EXPLORE_BASE: f32 = 19652.0;
const EXPLORE_INIT: f32 = 1.25;
const DIR_ALPHA: f32 = 0.3;
const DIR_EXPLORE_FRAC: f32 = 0.25;

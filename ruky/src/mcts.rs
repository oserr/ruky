// This module contains the components for the MCTS search.

use crate::board::Board;
use std::cell::RefCell;

#[derive(Debug)]
struct Node<'a> {
    board: Board,
    children: Vec<Node<'a>>,
    parent: Option<&'a Node<'a>>,
    prior: f32,
    init_value: f32,
    visits: RefCell<u32>,
    value: RefCell<f32>,
    is_leaf: RefCell<bool>,
}

// Creates a Node from a reference to a Board.
impl From<&Board> for Node<'_> {
    fn from(board: &Board) -> Self {
        Self {
            board: board.clone(),
            children: Vec::new(),
            parent: None,
            prior: 0.0,
            init_value: 0.0,
            visits: RefCell::new(1),
            value: RefCell::new(0.0),
            is_leaf: RefCell::new(true),
        }
    }
}

impl Node<'_> {
    // Creates a Node from a board and a prior.
    fn with_prior(board: &Board, prior: f32) -> Self {
        let mut node = Node::from(board);
        node.prior = prior;
        node
    }

    // Creates a Node from a board, a parent, and a prior.
    fn with_parent_and_prior<'a>(board: &Board, parent: &'a Node, prior: f32) -> Node<'a> {
        let mut node = Node::from(board);
        node.parent = Some(parent);
        node.prior = prior;
        node
    }

    fn choose_next(&self) -> Option<&Node> {
        self.children.iter().reduce(|acc_node, node| {
            let acc_node_uct = acc_node.mean_uct();
            let node_uct = node.mean_uct();
            if acc_node_uct > node_uct {
                acc_node
            } else {
                node
            }
        })
    }

    fn mean_uct(&self) -> f32 {
        *self.value.borrow() / *self.visits.borrow() as f32 + self.uct()
    }

    fn uct(&self) -> f32 {
        assert!(self.parent.is_some());
        let term1 = self.explore_rate() * self.prior;
        let term2 = (*self.parent.unwrap().visits.borrow() as f32).sqrt()
            / (1 + *self.visits.borrow()) as f32;
        term1 * term2
    }

    fn explore_rate(&self) -> f32 {
        assert!(self.parent.is_some());
        let num = 1.0 + *self.parent.unwrap().visits.borrow() as f32 + EXPLORE_BASE;
        (num / EXPLORE_BASE).log2() + EXPLORE_INIT
    }

    fn is_terminal(&self) -> bool {
        self.board.is_terminal()
    }
}

const EXPLORE_BASE: f32 = 19652.0;
const EXPLORE_INIT: f32 = 1.25;
const DIR_ALPHA: f32 = 0.3;
const DIR_EXPLORE_FRAC: f32 = 0.25;

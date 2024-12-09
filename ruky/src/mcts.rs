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
}

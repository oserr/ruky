// This module contains the components for the MCTS search.

use crate::board::Board;

#[derive(Debug)]
struct NodeTree {
    children: Vec<Node>,
}

impl NodeTree {
    fn choose_next(&self, parent_node: &Node) -> Option<&Node> {
        assert!(!parent_node.is_leaf);
        self.children[parent_node.children.0..parent_node.children.1+1]
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
    }

    fn root(&self) -> &Node {
        &self.children[0]
    }
}

#[derive(Debug)]
struct Node {
    board: Board,
    children: (usize, usize),
    parent: Option<usize>,
    prior: f32,
    init_value: f32,
    visits: u32,
    value: f32,
    is_leaf: bool,
}

// Creates a Node from a reference to a Board.
impl From<&Board> for Node {
    fn from(board: &Board) -> Self {
        Self {
            board: board.clone(),
            children: (0, 0),
            parent: None,
            prior: 0.0,
            init_value: 0.0,
            visits: 0,
            value: 0.0,
            is_leaf: true,
        }
    }
}

impl Node {
    // Creates a Node from a board and a prior.
    fn with_prior(board: &Board, prior: f32) -> Self {
        let mut node = Node::from(board);
        node.prior = prior;
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

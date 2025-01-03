use crate::board::Board;
use crate::err::RukyErr;
use crate::piece::Piece;
use crate::piece_move::PieceMove;
use std::time::Duration;

// A trait for evaluating a chess board position.
pub trait Search {
    // Computes the best possible move given a single board.
    fn search_board(&self, board: &Board) -> Result<SearchResult, RukyErr>;

    // Computes the best move given a series of moves, each move represented as a
    // full board position. Note that we don't need the game to evaluate a
    // position. It is assumed that the last Board represents the current position.
    fn search_game(&self, boards: &[Board]) -> Result<SearchResult, RukyErr> {
        self.search_board(boards.last().ok_or(RukyErr::SearchMissingBoard)?)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    // The best move according to the the Search agent.
    pub best: Bp,
    // The vector of probabilities for each move. This includes the best move.
    pub moves: Vec<Mp>,
    // The expected value from the best move.
    pub value: f32,
    // Total nodes expanded in search.
    pub nodes_expanded: u32,
    // Total nodes visited, including repeat visits.
    pub nodes_visited: u32,
    // The maximum depth of a branch explored during search.
    pub depth: u32,
    // Total time spent in eval mode - the component of the engine that computes the score for a
    // given position.
    pub total_eval_time: Duration,
    // Total time spent in search mode - includes eval mode + search time.
    pub total_search_time: Duration,
}

impl SearchResult {
    pub fn with_best(board: Board) -> Self {
        Self {
            best: Bp::with_board(board),
            moves: Vec::new(),
            value: 0.0,
            nodes_expanded: 0,
            nodes_visited: 0,
            depth: 0,
            total_eval_time: Duration::ZERO,
            total_search_time: Duration::ZERO,
        }
    }

    pub fn eval_time_per_expansion(&self) -> Duration {
        self.total_eval_time / self.nodes_expanded
    }

    pub fn eval_time_per_node(&self) -> Duration {
        self.total_eval_time / self.nodes_visited
    }

    pub fn search_time_per_expansion(&self) -> Duration {
        self.total_search_time / self.nodes_expanded
    }

    pub fn search_time_per_node(&self) -> Duration {
        self.total_search_time / self.nodes_visited
    }

    pub fn best_move(&self) -> Piece<PieceMove> {
        self.best.last_move()
    }

    pub fn best_board(&self) -> &Board {
        &self.best.board
    }
}

// Same as Bp, but only captures the move without the board.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mp {
    pub pm: Piece<PieceMove>,
    pub prior: f32,
    pub visits: u32,
}

// Packages together a board move with prior probability from the evaluator and
// the visit count from the MCTS.
#[derive(Clone, Debug, PartialEq)]
pub struct Bp {
    pub board: Board,
    pub prior: f32,
    pub visits: u32,
}

impl Bp {
    pub fn with_board(board: Board) -> Self {
        Self {
            board: board,
            prior: 0.0,
            visits: 0,
        }
    }

    pub fn last_move(&self) -> Piece<PieceMove> {
        self.board
            .last_move()
            .expect("Bp without last move is not valid.")
    }
}

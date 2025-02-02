use crate::board::Board;
use crate::err::RukyErr;
use crate::search::{Search, SearchResult};
use rand::{self, Rng};

pub struct RandomSearch;

impl RandomSearch {
    pub fn new() -> Self {
        Self {}
    }
}

impl Search for RandomSearch {
    fn search_board(&mut self, board: &Board) -> Result<SearchResult, RukyErr> {
        let mut boards = board.next_boards().ok_or(RukyErr::SearchTerminalBoard)?;
        let index = rand::rng().random_range(0..boards.len());
        let best = boards.swap_remove(index);
        Ok(SearchResult::with_best(best))
    }
}

// This module defines the Eng trait, which is the main trait exposed by this
// library for clients of the library to hook their engines with library.

use crate::err::UziErr;
use crate::guicmd::{Go, Pos};
use crate::opt::{Opponent, PosValueOpt};
use std::path::Path;

pub trait Eng {
    fn set_hash_table_size(&mut self, table_size: u64) -> Result<(), UziErr>;

    fn set_nalimov_path<T: AsRef<Path>>(&mut self, path: T) -> Result<(), UziErr>;

    fn set_nalimov_cache(&mut self, cache_size: u64) -> Result<(), UziErr>;

    fn enable_ponder(&mut self) -> Result<(), UziErr>;

    fn disable_ponder(&mut self) -> Result<(), UziErr>;

    fn enable_own_book(&mut self) -> Result<(), UziErr>;
    fn disable_own_book(&mut self) -> Result<(), UziErr>;

    fn set_multi_pv(&mut self, nlines: u64) -> Result<(), UziErr>;

    fn show_curr_line(&mut self, show_curr_line: bool) -> Result<(), UziErr>;

    fn limit_strength(&mut self, limit_strength: bool) -> Result<(), UziErr>;

    fn set_elo(&mut self, elo: u16) -> Result<(), UziErr>;

    fn enable_analysis(&mut self) -> Result<(), UziErr>;

    fn disable_analysis(&mut self) -> Result<(), UziErr>;

    fn set_shredder_bases_path<T: AsRef<Path>>(&mut self, path: T) -> Result<(), UziErr>;

    fn set_opponent(&mut self, opponent: &Opponent) -> Result<(), UziErr>;

    fn set_pos_val(&mut self, pos_val: &PosValueOpt) -> Result<(), UziErr>;

    fn set_position(&mut self, pos: &Pos) -> Result<(), UziErr>;

    fn go(&mut self, go_cmd: &Go) -> Result<(), UziErr>;

    fn stop(&mut self) -> Result<(), UziErr>;

    fn new_game(&mut self) -> Result<(), UziErr>;
}

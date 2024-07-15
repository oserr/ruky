// A helper class to hold the configuration for the engine, i.e what options are
// enabled and disabled.

use crate::guicmd::Pos;
use crate::opt::{Opponent, PosValueOpt};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Config {
    pub hash_table: Option<u64>,
    pub nalimov_path: Option<PathBuf>,
    pub nalimov_cache: Option<u64>,
    pub ponder: bool,
    pub own_book: bool,
    pub multi_pv: Option<u64>,
    pub show_curr_line: bool,
    pub limit_strength: bool,
    pub elo: Option<u16>,
    pub analysis_mode: bool,
    pub shredder_bases: Option<PathBuf>,
    pub opponent: Option<Opponent>,
    pub pos_value: Option<PosValueOpt>,
    pub pos: Option<Pos>,
}

impl Config {
    fn new() -> Self {
        Self::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hash_table: None,
            nalimov_path: None,
            nalimov_cache: None,
            ponder: false,
            own_book: false,
            multi_pv: None,
            show_curr_line: false,
            limit_strength: false,
            elo: None,
            analysis_mode: false,
            shredder_bases: None,
            opponent: None,
            pos_value: None,
            pos: None,
        }
    }
}

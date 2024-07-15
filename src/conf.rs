// A helper class to hold the configuration for the engine, i.e what options are
// enabled and disabled.

use crate::guicmd::Pos;
use crate::opt::{Opponent, PosValueOpt};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Config {
    hash_table: Option<u64>,
    nalimov_path: Option<PathBuf>,
    nalimov_cache: Option<u64>,
    ponder: bool,
    own_book: bool,
    multi_pv: Option<u64>,
    show_curr_line: bool,
    limit_strength: bool,
    elo: Option<u16>,
    analysis_mode: bool,
    shredder_bases: Option<PathBuf>,
    opponent: Option<Opponent>,
    pos_value: Option<PosValueOpt>,
    pos: Option<Pos>,
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

// A helper class to hold the configuration for the engine, i.e what options are
// enabled and disabled.

use crate::guicmd::Pos;
use crate::opt::{HasOpt, Opponent, PosValueOpt, UziOpt, UziOptIter};
use crate::types::{SpinType, StrType};
use std::path::PathBuf;

// Config represents the current configuration for the chess engine, i.e. what
// options are supported, their defaults, etc. An option that is None means that
// it is not supported by the chess engine.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Config {
    pub hash_table: Option<SpinType<u64>>,
    pub nalimov_path: Option<PathBuf>,
    pub nalimov_cache: Option<SpinType<u64>>,
    pub ponder: Option<bool>,
    pub own_book: Option<bool>,
    pub multi_pv: Option<u64>,
    pub show_curr_line: Option<bool>,
    pub show_refutations: Option<bool>,
    pub limit_strength: Option<bool>,
    pub elo: Option<SpinType<u16>>,
    pub analysis_mode: Option<bool>,
    pub shredder_bases: Option<PathBuf>,
    pub opponent: Option<Opponent>,
    pub pos_value: Option<PosValueOpt>,
    pub pos: Option<Pos>,
    pub about: Option<StrType>,
}

impl Config {
    fn new() -> Self {
        Self::default()
    }
}

struct ConfigIter<'a> {
    opt_iter: UziOptIter,
    conf: &'a Config,
}

impl<'a> ConfigIter<'a> {
    fn new(conf: &'a Config) -> Self {
        Self {
            opt_iter: UziOpt::Hash.into_iter(),
            conf,
        }
    }
}

impl Iterator for ConfigIter<'_> {
    type Item = HasOpt;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(opt) = self.opt_iter.next() {
            match opt {
                UziOpt::Hash if self.conf.hash_table.is_some() => {
                    return Some(HasOpt::Hash(self.conf.hash_table.unwrap().clone()));
                }
                UziOpt::NalimovPath if self.conf.nalimov_path.is_some() => {
                    let path = self
                        .conf
                        .nalimov_path
                        .clone()
                        .unwrap()
                        .into_os_string()
                        .into_string()
                        .unwrap();
                    return Some(HasOpt::NalimovPath(StrType(path)));
                }
                UziOpt::NalimovCache if self.conf.nalimov_cache.is_some() => {
                    return Some(HasOpt::NalimovCache(
                        self.conf.nalimov_cache.unwrap().clone(),
                    ));
                }
                UziOpt::Ponder if self.conf.ponder.is_some() => {
                    return Some(HasOpt::Ponder(self.conf.ponder.unwrap().into()));
                }
                UziOpt::OwnBook if self.conf.own_book.is_some() => {
                    return Some(HasOpt::OwnBook(self.conf.own_book.unwrap().into()));
                }
                UziOpt::MultiPv if self.conf.multi_pv.is_some() => {
                    todo!();
                }
                UziOpt::ShowCurrLine if self.conf.show_curr_line.is_some() => {
                    todo!();
                }
                UziOpt::ShowRefutations if self.conf.show_refutations.is_some() => {
                    todo!();
                }
                UziOpt::LimitStrength if self.conf.limit_strength.is_some() => {
                    todo!();
                }
                UziOpt::Elo if self.conf.elo.is_some() => {
                    todo!();
                }
                UziOpt::AnalysisMode if self.conf.analysis_mode.is_some() => {
                    todo!();
                }
                UziOpt::Opponent if self.conf.opponent.is_some() => {
                    todo!();
                }
                UziOpt::About if self.conf.about.is_some() => {
                    todo!();
                }
                UziOpt::ShredderBasesPath if self.conf.shredder_bases.is_some() => {
                    todo!();
                }
                UziOpt::SetPositionValue if self.conf.pos_value.is_some() => {
                    todo!();
                }
                _ => continue,
            };
        }
        None
    }
}

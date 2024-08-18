// A helper class to hold the configuration for the engine, i.e what options are
// enabled and disabled.

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
    pub multi_pv: Option<SpinType<u64>>,
    pub show_curr_line: Option<bool>,
    pub show_refutations: Option<bool>,
    pub limit_strength: Option<bool>,
    pub elo: Option<SpinType<u16>>,
    pub analysis_mode: Option<bool>,
    pub opponent: Option<Opponent>,
    pub about: Option<StrType>,
    pub shredder_bases: Option<PathBuf>,
    pub pos_value: Option<PosValueOpt>,
}

impl Config {
    fn new() -> Self {
        Self::default()
    }

    fn iter(&self) -> impl Iterator<Item = HasOpt> + '_ {
        ConfigIter::new(self)
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
                    return Some(HasOpt::Hash(self.conf.hash_table.unwrap()));
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
                    return Some(HasOpt::NalimovCache(self.conf.nalimov_cache.unwrap()));
                }
                UziOpt::Ponder if self.conf.ponder.is_some() => {
                    return Some(HasOpt::Ponder(self.conf.ponder.unwrap().into()));
                }
                UziOpt::OwnBook if self.conf.own_book.is_some() => {
                    return Some(HasOpt::OwnBook(self.conf.own_book.unwrap().into()));
                }
                UziOpt::MultiPv if self.conf.multi_pv.is_some() => {
                    return Some(HasOpt::MultiPv(self.conf.multi_pv.unwrap()));
                }
                UziOpt::ShowCurrLine if self.conf.show_curr_line.is_some() => {
                    return Some(HasOpt::ShowCurrLine(
                        self.conf.show_curr_line.unwrap().into(),
                    ));
                }
                UziOpt::ShowRefutations if self.conf.show_refutations.is_some() => {
                    return Some(HasOpt::ShowRefutations(
                        self.conf.show_refutations.unwrap().into(),
                    ));
                }
                UziOpt::LimitStrength if self.conf.limit_strength.is_some() => {
                    return Some(HasOpt::LimitStrength(
                        self.conf.limit_strength.unwrap().into(),
                    ));
                }
                UziOpt::Elo if self.conf.elo.is_some() => {
                    return Some(HasOpt::Elo(self.conf.elo.unwrap()));
                }
                UziOpt::AnalysisMode if self.conf.analysis_mode.is_some() => {
                    return Some(HasOpt::AnalysisMode(
                        self.conf.analysis_mode.unwrap().into(),
                    ));
                }
                UziOpt::Opponent if self.conf.opponent.is_some() => {
                    return Some(HasOpt::Opp(self.conf.opponent.clone().unwrap().into()));
                }
                UziOpt::About if self.conf.about.is_some() => {
                    return Some(HasOpt::About(self.conf.about.clone().unwrap()));
                }
                UziOpt::ShredderBasesPath if self.conf.shredder_bases.is_some() => {
                    let path = self
                        .conf
                        .shredder_bases
                        .clone()
                        .unwrap()
                        .into_os_string()
                        .into_string()
                        .unwrap();
                    return Some(HasOpt::ShredderBasesPath(StrType(path)));
                }
                UziOpt::SetPosVal if self.conf.pos_value.is_some() => {
                    return Some(HasOpt::SetPosVal(
                        self.conf.pos_value.clone().unwrap().into(),
                    ));
                }
                _ => continue,
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CheckType;
    use std::str::FromStr;

    #[test]
    fn conf_iter_no_values() {
        let conf = Config::new();
        let mut iter = conf.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn conf_iter_with_hash_table() {
        let spin = SpinType::<u64> {
            default: 1,
            min: 0,
            max: 2,
        };
        let mut conf = Config::new();
        conf.hash_table = Some(spin);
        let mut iter = conf.iter();
        assert_eq!(iter.next(), Some(HasOpt::Hash(spin)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn conf_iter_with_some_values() {
        let about = StrType("about".into());
        let some_path = "/some/path".to_string();

        let mut conf = Config::new();
        conf.nalimov_path = Some(PathBuf::from_str(&some_path).unwrap());
        conf.own_book = Some(true);
        conf.show_curr_line = Some(true);
        conf.about = Some(about.clone());

        let mut iter = conf.iter();

        assert_eq!(iter.next(), Some(HasOpt::NalimovPath(StrType(some_path))));
        assert_eq!(iter.next(), Some(HasOpt::OwnBook(CheckType(true))));
        assert_eq!(iter.next(), Some(HasOpt::ShowCurrLine(CheckType(true))));
        assert_eq!(iter.next(), Some(HasOpt::About(about)));
        assert_eq!(iter.next(), None);
    }
}

// This module defines the Eng trait, which is the main trait exposed by this
// library for clients of the library to hook their engines with library.

use crate::conf::Config;
use crate::engtx::EngOutTx;
use crate::err::UziErr;
use crate::guicmd::{Go, GuiCmd, Pos};
use crate::opt::{Opponent, PosValueOpt, SetOpt};
use crate::types::SpinType;
use std::cmp::PartialOrd;
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

pub trait Eng {
    fn hash_table_size(&mut self, table_size: u64) -> Result<(), UziErr>;
    fn nalimov_path(&mut self, patho: &Path) -> Result<(), UziErr>;
    fn nalimov_cache(&mut self, cache_size: u64) -> Result<(), UziErr>;
    fn ponder(&mut self, is_enabled: bool) -> Result<(), UziErr>;
    fn own_book(&mut self, is_enabled: bool) -> Result<(), UziErr>;
    fn multi_pv(&mut self, nlines: u64) -> Result<(), UziErr>;
    fn show_curr_line(&mut self, show_curr_line: bool) -> Result<(), UziErr>;
    fn show_refutations(&mut self, show_refutations: bool) -> Result<(), UziErr>;
    fn limit_strength(&mut self, limit_strength: bool) -> Result<(), UziErr>;
    fn elo(&mut self, elo: u16) -> Result<(), UziErr>;
    fn analysis(&mut self, is_enabled: bool) -> Result<(), UziErr>;
    fn shredder_bases(&mut self, path: &Path) -> Result<(), UziErr>;
    fn opponent(&mut self, opponent: &Opponent) -> Result<(), UziErr>;
    fn pos_val(&mut self, pos_val: &PosValueOpt) -> Result<(), UziErr>;
    fn position(&mut self, pos: &Pos) -> Result<(), UziErr>;
    fn go(&mut self, go_cmd: &Go) -> Result<(), UziErr>;
    fn stop(&mut self) -> Result<(), UziErr>;
    fn new_game(&mut self) -> Result<(), UziErr>;
    fn quit(&mut self) -> Result<(), UziErr>;
}

// Represents the current engine state.
// - Waiting - the engine is still waiting for the GUI to connect.
// - Connected - the GUI is connected but has not started a new game.
// - NewGame - The GUI has started a new game.
// - Go - The engine is analyzing a position.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EngState {
    Waiting,
    Connected,
    NewGame,
    Go,
    Quit,
}

impl EngState {
    pub fn is_waiting(&self) -> bool {
        matches!(self, EngState::Waiting)
    }

    pub fn is_connected_or_game(&self) -> bool {
        matches!(self, EngState::Connected | EngState::NewGame)
    }
}

// The Uzi [Eng]ine [Con]troller.
struct EngCon<E: Eng, O: EngOutTx> {
    eng: E,
    eng_out: Arc<O>,
    conf: Config,
    state: EngState,
}

impl<E: Eng, O: EngOutTx> EngCon<E, O> {
    pub fn create(eng: E, eng_out: Arc<O>, conf: Config) -> Self {
        Self {
            eng: eng,
            eng_out: eng_out,
            conf: conf,
            state: EngState::Waiting,
        }
    }

    pub fn run(&mut self) -> Result<(), UziErr> {
        for line in stdin().lines() {
            match line {
                // Here we should shut down gracefully and return an error.
                Err(_err) => todo!(),
                Ok(line) => {
                    let cmd = GuiCmd::from_str(&line);
                    match cmd {
                        // TODO: log the error.
                        Err(_) => continue,
                        Ok(cmd) => self.handle_cmd(cmd),
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: GuiCmd) {
        match cmd {
            GuiCmd::Uci if self.state.is_waiting() => {
                self.eng_out.send_name(self.conf.id_name.clone());
                self.eng_out.send_author(self.conf.id_author.clone());
                for opt in self.conf.iter() {
                    self.eng_out.send_opt(opt);
                }
                self.eng_out.send_uciok();
                self.state = EngState::Connected;
            }
            GuiCmd::IsReady => self.eng_out.send_ready(),
            GuiCmd::Debug(_is_enabled) => todo!(),
            GuiCmd::SetOpt(opt) => self.set_opt(opt),
            GuiCmd::Pos(_pos) => todo!(),
            GuiCmd::NewGame => todo!(),
            GuiCmd::Go(_go) => todo!(),
            GuiCmd::Stop => todo!(),
            GuiCmd::Ponderhit => todo!(),
            _ => (),
        }
    }

    fn set_opt(&mut self, opt: SetOpt) {
        match opt {
            SetOpt::Hash(table_size) => set_spin_opt(table_size, self.conf.hash_table, |x| {
                self.eng.hash_table_size(x)
            }),
            SetOpt::NalimovPath(path_buf) => {
                set_path(path_buf, self.conf.nalimov_path.is_some(), |x| {
                    self.eng.nalimov_path(x)
                })
            }
            SetOpt::NalimovCache(cache_size) => {
                set_spin_opt(cache_size, self.conf.nalimov_cache, |x| {
                    self.eng.nalimov_cache(x)
                })
            }
            SetOpt::Ponder(enabled) => {
                set_bool_opt(enabled, self.conf.ponder.is_some(), |x| self.eng.ponder(x))
            }
            SetOpt::OwnBook(enabled) => set_bool_opt(enabled, self.conf.own_book.is_some(), |x| {
                self.eng.own_book(x)
            }),
            SetOpt::MultiPv(k_best) => {
                set_spin_opt(k_best, self.conf.multi_pv, |x| self.eng.multi_pv(x))
            }
            SetOpt::ShowCurrLine(enabled) => {
                set_bool_opt(enabled, self.conf.show_curr_line.is_some(), |x| {
                    self.eng.show_curr_line(x)
                })
            }
            SetOpt::ShowRefutations(enabled) => {
                set_bool_opt(enabled, self.conf.show_refutations.is_some(), |x| {
                    self.eng.show_refutations(x)
                })
            }
            SetOpt::LimitStrength(enabled) => {
                set_bool_opt(enabled, self.conf.limit_strength.is_some(), |x| {
                    self.eng.limit_strength(x)
                })
            }
            SetOpt::Elo(elo) => set_spin_opt(elo, self.conf.elo, |x| self.eng.elo(x)),
            SetOpt::AnalysisMode(enabled) => {
                set_bool_opt(enabled, self.conf.analysis_mode.is_some(), |x| {
                    self.eng.analysis(x)
                })
            }
            SetOpt::ShredderBasesPath(path_buf) => {
                set_path(path_buf, self.conf.shredder_bases.is_some(), |x| {
                    self.eng.shredder_bases(x)
                })
            }
            SetOpt::Opp(_opponent) => todo!(),
            SetOpt::SetPosVal(_pos_val) => todo!(),
        }
    }
}

fn set_path<F>(path_buf: PathBuf, is_supported: bool, mut setter_fn: F)
where
    F: FnMut(&Path) -> Result<(), UziErr>,
{
    if !is_supported {
        // TODO: log that feature is not supported.
        return;
    }

    if let Err(_) = setter_fn(path_buf.as_ref()) {
        // TODO: Log some error here.
        return;
    }
}

fn set_bool_opt<F>(enabled: bool, is_supported: bool, mut setter_fn: F)
where
    F: FnMut(bool) -> Result<(), UziErr>,
{
    if !is_supported {
        // TODO: log that feature is not supported.
        return;
    }

    if let Err(_) = setter_fn(enabled) {
        // TODO: Log some error here.
        return;
    }
}

fn set_spin_opt<T, F>(val: T, spin_val: Option<SpinType<T>>, mut setter_fn: F)
where
    T: PartialOrd,
    F: FnMut(T) -> Result<(), UziErr>,
{
    match spin_val {
        // Log that option is not enabled.
        None => todo!(),
        Some(ref val_opts) => {
            if val < val_opts.min || val > val_opts.max {
                // TODO: Log that value is out of range.
                return;
            }
            if let Err(_) = setter_fn(val) {
                // TODO: Log some error here.
                return;
            }
        }
    }
}

// This module defines the Eng trait, which is the main trait exposed by this
// library for clients of the library to hook their engines with library.

use crate::conf::Config;
use crate::engtx::EngOutTx;
use crate::err::UziErr;
use crate::guicmd::{Go, GuiCmd, Pos};
use crate::opt::{Opponent, PosValueOpt};
use std::io::stdin;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

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
            GuiCmd::Uci if self.state == EngState::Waiting => {
                self.eng_out.send_name(self.conf.id_name.clone());
                self.eng_out.send_author(self.conf.id_author.clone());
                for opt in self.conf.iter() {
                    self.eng_out.send_opt(opt);
                }
                self.eng_out.send_uciok();
                self.state = EngState::Connected;
            }
            GuiCmd::IsReady => todo!(),
            GuiCmd::Debug(_is_enabled) => todo!(),
            GuiCmd::SetOpt(_opt) => todo!(),
            GuiCmd::Pos(_pos) => todo!(),
            GuiCmd::NewGame => todo!(),
            GuiCmd::Go(_go) => todo!(),
            GuiCmd::Stop => todo!(),
            GuiCmd::Ponderhit => todo!(),
            _ => (),
        }
    }
}

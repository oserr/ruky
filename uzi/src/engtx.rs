// This module implements abstractions to transmit messages from the chess
// engine to the GUI.

use crate::engcmd::{EngCmd, Info};
use crate::opt::HasOpt;
use crate::pm::Pm;
use std::io::{stdout, Write};
use std::sync::mpsc::{channel, Sender};
use std::thread;

// A trait for tramitting commands from the communcation protocol engine to the
// GUI.
pub trait EngOutTx: EngTx {
    fn send_name(&self, name: String);
    fn send_author(&self, author: String);
    fn send_uciok(&self);
    fn send_ready(&self);
    fn send_opt(&self, opt: HasOpt);
}

// A trait for tramitting commands to the GUI. This is injected into the chess
// engine so it can transmit best moves and info messages to the GUI. Note that
// we don't want to inject EngOutTx into the engine, because the Uzi library
// should handle as much of the protocl as possible. Info data and moves are
// computed by the actual chess engine, and hence these need to be sent by the
// engine.
pub trait EngTx {
    fn send_best(&self, best: Pm);
    fn send_ponder(&self, best: Pm, ponder: Pm);
    fn send_info(&self, info: Info);
}

// This is the default impl for EngOutTx provided by the library.
#[derive(Clone, Debug)]
pub struct UziOut {
    sender: Sender<EngCmd>,
}

// Default initialization for UziOut. It spanws a thread to handle all writing
// to stdout. It creates a channel -- the receiving part is used in the spawned
// thread to send messages to the GUI, and the sender part is used by UziOut to
// enque the messages that need to be sent to the GUI.
impl Default for UziOut {
    fn default() -> Self {
        let (sender, receiver) = channel::<EngCmd>();
        let mut fout = stdout();
        thread::spawn(move || {
            for eng_cmd in receiver.iter() {
                let buffer = format!("{}\n", eng_cmd);
                if let Err(_) = fout.write_all(buffer.as_ref()) {
                    log::error!("Unable to send message [{}] to GUI.", buffer);
                }
            }
        });
        Self { sender }
    }
}

impl UziOut {
    pub fn new() -> Self {
        UziOut::default()
    }

    fn send_cmd(&self, cmd: EngCmd) {
        if let Err(cmd) = self.sender.send(cmd) {
            log::error!(
                "Unable to send message [{}] via Sender -- channel is broken.",
                cmd
            );
        }
    }
}

impl EngOutTx for UziOut {
    fn send_name(&self, name: String) {
        self.send_cmd(EngCmd::IdName(name));
    }

    fn send_author(&self, author: String) {
        self.send_cmd(EngCmd::IdAuthor(author));
    }

    fn send_uciok(&self) {
        self.send_cmd(EngCmd::UciOk);
    }

    fn send_ready(&self) {
        self.send_cmd(EngCmd::ReadyOk);
    }

    fn send_opt(&self, opt: HasOpt) {
        self.send_cmd(EngCmd::HasOpt(opt));
    }
}

impl EngTx for UziOut {
    fn send_best(&self, best: Pm) {
        self.send_cmd(EngCmd::BestMove {
            best: best,
            ponder: None,
        });
    }

    fn send_ponder(&self, best: Pm, ponder: Pm) {
        self.send_cmd(EngCmd::BestMove {
            best: best,
            ponder: Some(ponder),
        });
    }

    fn send_info(&self, info: Info) {
        self.send_cmd(EngCmd::Info(info));
    }
}

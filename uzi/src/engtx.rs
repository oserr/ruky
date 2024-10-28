// This module implements abstractions to transmit messages from the chess
// engine to the GUI.

use crate::engcmd::{EngCmd, Info};
use crate::opt::HasOpt;
use crate::pm::Pm;
use std::sync::Arc;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::runtime::Runtime;

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
struct UziOut {
    run_time: Arc<Runtime>,
}

impl UziOut {
    fn send_cmd(&self, cmd: EngCmd) {
        self.run_time.spawn(async move {
            let result = stdout().write(cmd.to_string().as_bytes()).await;
            if let Err(_) = result {
                todo!();
            }
        });
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

impl From<Arc<Runtime>> for UziOut {
    fn from(run_time: Arc<Runtime>) -> Self {
        Self { run_time }
    }
}

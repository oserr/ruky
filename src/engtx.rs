// This module implements abstractions to transmit messages from the chess
// engine to the GUI.

use crate::engcmd::Info;
use crate::opt::HasOpt;
use crate::pm::Pm;
use std::sync::Arc;
use tokio::runtime::Runtime;

// A trait for tramitting commands from the communcation protocol engine to the
// GUI.
pub trait EngOutTx {
    fn send_name(&self, name: String);
    fn send_author(&self, name: String);
    fn send_uciok(&self);
    fn send_ready(&self);
    fn send_best(&self, best: Pm);
    fn send_ponder(&self, best: Pm, ponder: Pm);
    fn send_info(&self, info: Info);
    fn send_opt(&self, opt: HasOpt);
}

// A trait for tramitting commands to the GUI. This is injected into the chess
// engine so it can transmit best moves and info messages to the GUI.
pub trait EngTx {
    fn send_best(&self, best: Pm);
    fn send_ponder(&self, best: Pm, ponder: Pm);
    fn send_info(&self, info: Info);
}

#[derive(Clone, Debug)]
struct UziEngOut {
    run_time: Arc<Runtime>,
}

impl From<Arc<Runtime>> for UziEngOut {
    fn from(run_time: Arc<Runtime>) -> Self {
        Self { run_time }
    }
}

// This module contains the types to represent commands from the chess engine to
// a GUI.

use crate::opt::HasOpt;
use crate::pm::Pm;

// Represents a command from the engine to the GUI.
// TODO: support copyprotection, registration, and custom commands:
// - copyprotection: Used by copyprotected engines.
// - registration [ok | error]: Needed for engines that need a username and or a
//   code to function
// with all the features.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EngCmd {
    // id name <x>: The name and version of the chess engine, as response to "uci" command..
    IdName(String),
    // id author <x>: The name of the author of the chess engine, as response to "uci" command.
    IdAuthor(String),
    // uciok: Must be sent after the ID and optional options to tell the GUI that the engine has
    // sent all infos and is ready in uci mode.
    UciOk,
    // readyok: This must be sent when the engine has received an "isready" command and has
    // processed all input and is ready to accept new commands now.
    ReadyOk,
    // bestmove <move1> [ponder <move2>]: The engine has stopped searching and found the move
    // <move1> best in this position. The engine can send the move it likes to ponder on. The
    // engine must not start pondering automatically. This command must always be sent if the
    // engine stops searching, also in pondering mode if there is a "stop" command, so for
    // every "go" command a "bestmove" command is needed. Directly before that, the engine
    // should send a final info command with the final search information.
    BestMove {
        best: Pm,
        ponder: Option<Pm>,
    },
    // info [opts]: Used by the engine to send information about the engine and its calculations
    // to the GUI. See below for more details.
    Info(Info),
    // option name <id> [opts..]: To tell the engine which options can be changed.
    HasOpt(HasOpt),
}

// Represents the various options to encode the "info" command, when the engine
// wants to send information to the GUI. This should be done whenever one of the
// info has changed. The engine can send only selected infos or mutliple infos
// with one info command, e.g. "info currmove e2e4 currmovenumber 1", "info
// depth 12 nodes 123456 nps 1000000". All infos belonging to the pv should be
// sent together, e.g. "info depth 2 score cp 214 time 1242 nodes 2124 nps 34928
// pv e2e4 e7e5 g1f3". Suggest to send "currmove", "currmovenumber", "currline",
// and "refutation" only after 1 second to avoid too much traffic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Info {
    // depth <x>: Search depth in plies.
    depth: Option<u16>,

    // seldepth <x>: Selective search depth in plies. If the engine sends "seldepth", there must
    // also be a "depth" present in the same string.
    sel_depth: Option<u16>,

    // time <x>: The time searched in ms. This should be sent together with the PV.
    time: Option<u32>,

    // node <x>: x nodes searched. The engine should send this info regularly.
    node: Option<u32>,

    // pv <move1> .. <movei>: The best line found.
    pv: Option<Vec<Pm>>,

    // multipv <num>: This for the multipv mode. For the best move/pv add "multipv 1" in the string
    // when you send the pv. In k-best mode always send the all k variants in k strings
    // together.
    multi_pv: Option<MultiPv>,

    // score [opts]: The score from the engine's point of view.
    score: Option<Score>,

    // currmove <move>: Currently searching this move.
    curr_move: Option<Pm>,

    // hashfull <x>: The hashfull is x permill full. The engine should send this info regularly.
    hash_full: Option<u16>,

    // nps <x>: x nodes per second searched. The engine should send this info regularly.
    nodes_per_sec: Option<u32>,

    // tbhits <x>: x positions where found in the endgame table base.
    tb_hits: Option<u32>,

    // sbhits <x>: x positions where found in the shredder endgame databases.
    sb_hits: Option<u32>,

    // string <str>: Any string <str> which will be displayed by the engine. If there is a string
    // command the rest of the line will be interpreted as <str>.
    cpu_load: Option<u16>,

    // refutation <move1> <move2> .. <movei>: move1 is refuted by line.
    refutation: Option<Refutation>,

    // currline <cpunr> <move1> .. <movei>: The current line the engine is calculating. <cpnur> is
    // only relevant if more than one CPU is used. See CurrentLine for more detaisl.
    curr_line: Option<CurrentLine>,
}

// currline <cpunr> <move1> .. <movei>: Represents the current line the engine
// is calculating. <cpunr> is the number of the cpu if the   engine is running
// on more than one cpu. <cpunr> = 1, 2, 3, etc. If the engien is just using one
// CPU, <cpunr> can be omitted. If <cpunr> is greater than 1, always send all
// k lines in k strings   together. The engine should only send this if the
// option "UCI_ShowCurrLine" is set to true.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CurrentLine {
    cpu_id: Option<u16>,
    line: Vec<Pm>,
}

// refutation <move1> <move2> .. <movei>: Represents the refutation command.
// Move <move1> is refuted by the line <move2> .. <movei>. The engine should
// only send this if the option "UCI_ShowRefutations" is set to true.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Refutation {
    // The move being refuted.
    mv: String,

    // The line of moves that refute move |mv|.
    moves: Vec<Pm>,
}

// score cp <x> [mate <y>] [lowerbound] [upperbound]: Represents the score
// option to the info command.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Score {
    // cp <x>: The score from the engine's point of view in centipawns.
    cp: i32,

    // mate <y>: Mate in y moves, not plies. If the engine is getting mated, use negative values
    // for y.
    mate: Option<i16>,

    // If provided, then the score is either a lower or an upper bound.
    bound: Option<ScoreBound>,
}

// Represents a lower or an upper score bound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScoreBound {
    Lower,
    Upper,
}

// multipv <num>: Used for representing the multipv command in the multipv mode.
// For the best move/pv add "multipv 1" in the string when you send the pv. In
// k-best mode, should always send the all k variants in k strings together.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MultiPv {
    rank: u64,
    moves: Vec<Pm>,
}

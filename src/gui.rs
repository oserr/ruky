// This module contains artifacts used to build and represent commands from the
// GUI to the engine.

use crate::opt::UciOpt;

// Represents a command from the GUI to the engine.
pub enum GuiCommand {
    // uci: Tells the engine to switch to UCI mode.
    Uci,

    // debug: If true, then debug mode is enabled, otherwise it is disabled.
    Debug(bool),

    // isready: Used to synchronize the GUI with the engine. The command always needs to be
    // answered with readyok. If the engine is calculating, it should also send readyok without
    // stopping the calculation.
    IsReady,

    // setoption name <id> [value <x>]
    // This is sent to the engine when the user wants to change the internal parameters of the
    // engine. One command will be sent for each parameter and this will only be sent when the
    // engine is waiting.
    SetOpt(UciOpt),

    // ucinewgame: Sent to the engine when the next search, started with position and go will be
    // from a different game.
    NewGame,

    // position [fen <fenstring> | startpos] moves <move1> ... <movei>: A command to set up the
    // initial position.
    Pos(Position),

    // go [opts]: A command to tell the engine to begin calculating the best move.
    Go(Go),

    // stop: A command to tell the engine to stop calculating as soon as possible.
    Stop,

    // ponderhit: The user has played the expected move. This will be sent if the engine was told
    // to ponder on the same move the engine has played. The engine has switched from
    // pondering to normal search.
    Ponderhit,
}

// A struct to represent the UCI "go" command, used to tell the engine to begin
// calculating the best move given an intial position. The command can take
// multiple options. Start calculating on the current position set up with the
// "position" command.
pub struct Go {
    // searchmoves <move1> ... <movei>: Restricts calculation by one or more moves.
    search_moves: Option<Vec<String>>,

    // Starts searching in pondering mode.
    ponder: Option<bool>,

    // wtime <x>: White has x milliseconds on the clock.
    wtime: Option<u32>,

    // btime <x>: Black has x milliseconds on the clock.
    btime: Option<u32>,

    // winc <x>: White increment per move in milliseconds.
    winc: Option<u16>,

    // binc <x>: Black increment per move in milliseconds.
    binc: Option<u16>,

    // movestogo <x>: There are x moves to the next time control. If this is not set, then wtime
    // and btime represent sudden death.
    moves_to_go: Option<u16>,

    // depth <x>: Search x plies only.
    depth: Option<u16>,

    // nodes <x>: Search x nodes only.
    nodes: Option<u64>,

    // mate <x>: Search for a mate in x moves.
    mate: Option<u16>,

    // movetime <x>: Search exactly x milliseconds.
    move_time: Option<u32>,

    // infinite: Search until the stop command. Do not exit search without being told to do so in
    // this mode.
    infinite: Option<()>,
}

// A structure to represent the UCI "position" command, which is issued to the
// engine to set up the initial position.
// position [fen <fenstring> | startpos] moves <move1> ... <movei>
// Set up the position described in fenstring or from the starting position and
// play the moves. No new command is needed, but if the position is from a
// different game than the last position sent to the engine, then the GUI should
// have sent a "ucinewgame" in between.
pub struct Position {
    // Represents the initial position: either a new game or a FEN string.
    pos: PosOpt,

    // Moves to apply to the initial position. If set, the intial position is derived after the
    // moves are applied to the initial position.
    moves: Option<Vec<String>>,
}

// An enum to represent the two different types of positions that can be set,
// i.e. startpos or a position from a FEN string.
pub enum PosOpt {
    StartPos,
    Fen(String),
}

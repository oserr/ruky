// This module contains artifacts used to build and represent commands from the
// GUI to the engine.

use crate::err::UziErr;
use crate::opt::UciOpt;

// Represents a command from the GUI to the engine.
#[derive(Clone, Debug, PartialEq)]
pub enum GuiCmd {
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
    Pos(Pos),

    // go [opts]: A command to tell the engine to begin calculating the best move.
    Go(Go),

    // stop: A command to tell the engine to stop calculating as soon as possible.
    Stop,

    // ponderhit: The user has played the expected move. This will be sent if the engine was told
    // to ponder on the same move the engine has played. The engine has switched from
    // pondering to normal search.
    Ponderhit,
}

impl TryFrom<&str> for GuiCmd {
    type Error = UziErr;

    fn try_from(cmd: &str) -> Result<GuiCmd, Self::Error> {
        let words = cmd.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            return Err(UziErr::MissingCmd);
        }
        match words[0] {
            "uci" => Ok(GuiCmd::Uci),
            "isready" => Ok(GuiCmd::IsReady),
            "ucinewgame" => Ok(GuiCmd::NewGame),
            "stop" => Ok(GuiCmd::Stop),
            "ponderhit" => Ok(GuiCmd::Ponderhit),
            "debug" => match words[1] {
                "on" => Ok(GuiCmd::Debug(true)),
                "off" => Ok(GuiCmd::Debug(false)),
                _ => Err(UziErr::MissingOnOff),
            },
            "setoption" => todo!(),
            "position" => todo!(),
            "go" => todo!(),
            _ => Err(UziErr::What),
        }
    }
}

// A struct to represent the UCI "go" command, used to tell the engine to begin
// calculating the best move given an intial position. The command can take
// multiple options. Start calculating on the current position set up with the
// "position" command.
#[derive(Clone, Debug, PartialEq)]
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

// Default initialization for Go.
impl Default for Go {
    fn default() -> Self {
        Self {
            search_moves: None,
            ponder: None,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            moves_to_go: None,
            depth: None,
            nodes: None,
            mate: None,
            move_time: None,
            infinite: None,
        }
    }
}

impl Go {
    #[inline]
    pub fn new() -> Self {
        Go::default()
    }
}

// A Go command builder.
#[derive(Debug, Clone)]
struct GoBuilder {
    go: Go,
}

impl GoBuilder {
    pub fn new() -> Self {
        Self { go: Go::new() }
    }
}

// A Pos builder.
#[derive(Debug, Clone)]
pub struct PosBuilder {
    pos: Option<PosOpt>,
    moves: Option<Vec<String>>,
}

impl PosBuilder {
    // Creates a new builder with the Pos completely unset.
    pub fn new() -> Self {
        Self {
            pos: None,
            moves: None,
        }
    }

    // Initializes the position with a new game.
    pub fn start(&mut self) -> &mut Self {
        self.pos.replace(PosOpt::StartPos);
        self
    }

    // The initial position is taken from a FEN string.
    pub fn fen(&mut self, fen: &str) -> &mut Self {
        self.pos.replace(PosOpt::Fen(fen.into()));
        self
    }

    // Adds a move to the position. Moves should be added in the order they are
    // played.
    pub fn add_move(&mut self, mv: &str) -> &mut Self {
        if let Some(ref mut moves) = self.moves {
            moves.push(mv.into());
        } else {
            self.moves = Some(vec![mv.into()]);
        }
        self
    }

    // If the initial position is not set, returns an error.
    pub fn build(&mut self) -> Result<Pos, UziErr> {
        if self.pos.is_none() {
            return Err(UziErr::Position);
        }

        Ok(Pos {
            pos: self.pos.take().unwrap(),
            moves: self.moves.take(),
        })
    }
}

// A structure to represent the UCI "position" command, which is issued to the
// engine to set up the initial position.
// position [fen <fenstring> | startpos] moves <move1> ... <movei>
// Set up the position described in fenstring or from the starting position and
// play the moves. No new command is needed, but if the position is from a
// different game than the last position sent to the engine, then the GUI should
// have sent a "ucinewgame" in between.
#[derive(Clone, Debug, PartialEq)]
pub struct Pos {
    // Represents the initial position: either a new game or a FEN string.
    pos: PosOpt,

    // Moves to apply to the initial position. If set, the intial position is derived after the
    // moves are applied to the initial position.
    moves: Option<Vec<String>>,
}

impl TryFrom<&Vec<&str>> for Pos {
    type Error = UziErr;

    fn try_from(cmd: &Vec<&str>) -> Result<Pos, Self::Error> {
        let mut builder = PosBuilder::new();
        let mut pos_state = PosState::Begin;

        for word in cmd {
            match *word {
                "position" => {
                    if pos_state != PosState::Begin {
                        return Err(UziErr::Position);
                    }
                    pos_state = PosState::Position;
                }
                "fen" => {
                    if pos_state != PosState::Position {
                        return Err(UziErr::Position);
                    }
                    pos_state = PosState::Fen
                }
                "startpos" => {
                    if pos_state != PosState::Position {
                        return Err(UziErr::Position);
                    }
                    pos_state = PosState::StartPos;
                    builder.start();
                }
                "moves" => {
                    if pos_state != PosState::FenStr || pos_state != PosState::StartPos {
                        return Err(UziErr::Position);
                    }
                    pos_state = PosState::Moves;
                }
                _ => {
                    if pos_state == PosState::Fen {
                        pos_state = PosState::FenStr;
                        builder.fen(*word);
                    } else if pos_state == PosState::Moves {
                        builder.add_move(*word);
                    } else {
                        return Err(UziErr::Position);
                    }
                }
            };
        }

        match pos_state {
            PosState::FenStr | PosState::Moves | PosState::StartPos => builder.build(),
            _ => Err(UziErr::Position),
        }
    }
}

// Represents state of parsing the "position" command.
#[derive(PartialEq)]
enum PosState {
    Begin,
    Position,
    StartPos,
    Fen,
    FenStr,
    Moves,
}

// An enum to represent the two different types of positions that can be set,
// i.e. startpos or a position from a FEN string.
#[derive(Clone, Debug, PartialEq)]
pub enum PosOpt {
    StartPos,
    Fen(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pos_builder() {
        let pos_builder = PosBuilder::new();
        assert!(pos_builder.pos.is_none());
        assert!(pos_builder.moves.is_none());
    }

    #[test]
    fn pos_builder_with_startpos() {
        let mut pos_builder = PosBuilder::new();
        let pos = pos_builder
            .start()
            .build()
            .expect("Should build with startpos");
        assert_eq!(pos.pos, PosOpt::StartPos);
        assert!(pos.moves.is_none());
    }

    #[test]
    fn pos_builder_with_fen() {
        let mut pos_builder = PosBuilder::new();
        let pos = pos_builder
            .fen("fen")
            .build()
            .expect("Should build with fen");
        assert_eq!(pos.pos, PosOpt::Fen("fen".into()));
        assert!(pos.moves.is_none());
    }

    #[test]
    fn pos_builder_with_startpos_and_moves() {
        let mut pos_builder = PosBuilder::new();
        let pos = pos_builder
            .start()
            .add_move("e2e4")
            .add_move("e7e5")
            .build()
            .expect("Should build with startpos and moves");
        assert_eq!(pos.pos, PosOpt::StartPos);
        assert_eq!(pos.moves, Some(vec!["e2e4".into(), "e7e5".into()]));
    }
}

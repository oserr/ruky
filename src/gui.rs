// This module contains artifacts used to build and represent commands from the
// GUI to the engine.

use crate::err::UziErr;
use crate::opt::UciOpt;
use std::time::Duration;

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
            "debug" => {
                if words.len() <= 1 {
                    return Err(UziErr::MissingOnOff);
                }
                match words[1] {
                    "on" => Ok(GuiCmd::Debug(true)),
                    "off" => Ok(GuiCmd::Debug(false)),
                    _ => Err(UziErr::MissingOnOff),
                }
            }
            "setoption" => todo!(),
            "position" => Ok(GuiCmd::Pos(Pos::try_from(&words)?)),
            "go" => todo!(),
            _ => Err(UziErr::What),
        }
    }
}

// A helper builder class for the Go command.
#[derive(Clone, Debug, PartialEq)]
pub struct GoBuilder {
    go: Go,
}

impl GoBuilder {
    pub fn new() -> Self {
        Self { go: Go::new() }
    }

    // Adds a search move to restrict search.
    pub fn add_search_move(&mut self, mv: &str) -> &mut Self {
        if let Some(ref mut moves) = self.go.search_moves {
            moves.push(mv.into());
        } else {
            self.go.search_moves = Some(vec![mv.into()]);
        }
        self
    }

    // Sets the ponder mode.
    pub fn ponder(&mut self) -> &mut Self {
        self.go.ponder.replace(());
        self
    }

    // Remaining time for white in seconds.
    pub fn wtime_secs(&mut self, secs: u64) -> &mut Self {
        self.go.wtime.replace(Duration::from_secs(secs));
        self
    }

    // Remaining time for black in seconds.
    pub fn btime_secs(&mut self, secs: u64) -> &mut Self {
        self.go.btime.replace(Duration::from_secs(secs));
        self
    }

    // Time increment for white in seconds.
    pub fn winc_secs(&mut self, secs: u64) -> &mut Self {
        self.go.winc.replace(Duration::from_secs(secs));
        self
    }

    // Time increment for black in seconds.
    pub fn binc_secs(&mut self, secs: u64) -> &mut Self {
        self.go.binc.replace(Duration::from_secs(secs));
        self
    }

    // Sets the number of moves until the next time control.
    pub fn moves_to_go(&mut self, moves_to_go: u16) -> &mut Self {
        self.go.moves_to_go.replace(moves_to_go);
        self
    }

    // Sets the depth limit in plies.
    pub fn depth(&mut self, depth: u16) -> &mut Self {
        self.go.depth.replace(depth);
        self
    }

    // Max total nodes to search.
    pub fn nodes(&mut self, nodes: u64) -> &mut Self {
        self.go.nodes.replace(nodes);
        self
    }

    // Moves to force mate.
    pub fn mate(&mut self, mate: u16) -> &mut Self {
        self.go.mate.replace(mate);
        self
    }

    // Number of seconds to spend for the move.
    pub fn move_time_secs(&mut self, secs: u64) -> &mut Self {
        self.go.move_time.replace(Duration::from_secs(secs));
        self
    }

    // Search until the "stop" command is sent.
    pub fn infinite(&mut self) -> &mut Self {
        self.go.infinite.replace(());
        self
    }

    pub fn build(&mut self) -> Result<Go, UziErr> {
        if self.go.search_moves.is_none()
            && self.go.ponder.is_none()
            && self.go.wtime.is_none()
            && self.go.btime.is_none()
            && self.go.winc.is_none()
            && self.go.binc.is_none()
            && self.go.moves_to_go.is_none()
            && self.go.depth.is_none()
            && self.go.nodes.is_none()
            && self.go.mate.is_none()
            && self.go.move_time.is_none()
            && self.go.infinite.is_none()
        {
            return Err(UziErr::NothingSetForGo);
        }

        Ok(Go {
            search_moves: self.go.search_moves.take(),
            ponder: self.go.ponder.take(),
            wtime: self.go.wtime.take(),
            btime: self.go.btime.take(),
            winc: self.go.winc.take(),
            binc: self.go.binc.take(),
            moves_to_go: self.go.moves_to_go.take(),
            depth: self.go.depth.take(),
            nodes: self.go.nodes.take(),
            mate: self.go.mate.take(),
            move_time: self.go.move_time.take(),
            infinite: self.go.infinite.take(),
        })
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

    // ponder: Starts searching in pondering mode.
    ponder: Option<()>,

    // wtime <x>: White has x milliseconds on the clock.
    wtime: Option<Duration>,

    // btime <x>: Black has x milliseconds on the clock.
    btime: Option<Duration>,

    // winc <x>: White increment per move in milliseconds.
    winc: Option<Duration>,

    // binc <x>: Black increment per move in milliseconds.
    binc: Option<Duration>,

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
    move_time: Option<Duration>,

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

impl Pos {
    pub fn new() -> Self {
        Pos::default()
    }

    pub fn with_fen(fen: &str) -> Self {
        Pos {
            pos: PosOpt::Fen(fen.into()),
            moves: None,
        }
    }

    pub fn set_fen(&mut self, fen: &str) -> &mut Self {
        self.pos = PosOpt::Fen(fen.into());
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
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            pos: PosOpt::StartPos,
            moves: None,
        }
    }
}

impl TryFrom<&Vec<&str>> for Pos {
    type Error = UziErr;

    fn try_from(cmd: &Vec<&str>) -> Result<Pos, Self::Error> {
        let mut pos = Pos::new();
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
                }
                "moves" => {
                    if pos_state != PosState::FenStr && pos_state != PosState::StartPos {
                        return Err(UziErr::Position);
                    }
                    pos_state = PosState::Moves;
                }
                _ => {
                    if pos_state == PosState::Fen {
                        pos_state = PosState::FenStr;
                        pos.set_fen(*word);
                    } else if pos_state == PosState::Moves {
                        pos.add_move(*word);
                    } else {
                        return Err(UziErr::Position);
                    }
                }
            };
        }

        match pos_state {
            PosState::FenStr | PosState::Moves | PosState::StartPos => Ok(pos),
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
    fn pos_new() {
        let pos = Pos::new();
        assert_eq!(pos.pos, PosOpt::StartPos);
        assert!(pos.moves.is_none());
    }

    #[test]
    fn pos_with_fen() {
        let pos = Pos::with_fen("fen");
        assert_eq!(pos.pos, PosOpt::Fen("fen".into()));
        assert!(pos.moves.is_none());
    }

    #[test]
    fn pos_with_moves() {
        let mut pos = Pos::new();
        pos.add_move("e2e4").add_move("e7e5");
        assert_eq!(pos.pos, PosOpt::StartPos);
        assert_eq!(pos.moves, Some(vec!["e2e4".into(), "e7e5".into()]));
    }

    #[test]
    fn pos_try_from_empty_vec() {
        let args = vec![];
        assert_eq!(Pos::try_from(&args), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_with_only_position() {
        let args = vec!["position"];
        assert_eq!(Pos::try_from(&args), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_pos_with_startpos() {
        let args = vec!["position", "startpos"];
        assert_eq!(Pos::try_from(&args), Ok(Pos::new()));
    }

    #[test]
    fn pos_try_from_fen_but_missing_fen_string() {
        let args = vec!["position", "fen"];
        assert_eq!(Pos::try_from(&args), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_with_ranom_string() {
        let args = vec!["position", "random", "fen"];
        assert_eq!(Pos::try_from(&args), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_fen_with_fen() {
        let args = vec!["position", "fen", "FENSTRING"];
        assert_eq!(Pos::try_from(&args), Ok(Pos::with_fen("FENSTRING")));
    }

    #[test]
    fn pos_try_from_fen_with_moves() {
        let args = vec!["position", "fen", "FENSTRING", "moves", "e2e4", "e7e5"];
        let mut pos = Pos::with_fen("FENSTRING");
        pos.add_move("e2e4").add_move("e7e5");
        assert_eq!(Pos::try_from(&args), Ok(pos));
    }

    #[test]
    fn pos_try_from_startpos_with_moves() {
        let args = vec!["position", "startpos", "moves", "e2e4", "e7e5"];
        let mut pos = Pos::new();
        pos.add_move("e2e4").add_move("e7e5");
        assert_eq!(Pos::try_from(&args), Ok(pos));
    }

    #[test]
    fn pos_try_from_mixed_fen_and_startpos() {
        let args = vec!["position", "startpos", "fen"];
        assert_eq!(Pos::try_from(&args), Err(UziErr::Position));
    }
}

// This module contains artifacts used to build and represent commands from the
// GUI to the engine.

use crate::conv::{to_millis, to_number};
use crate::err::UziErr;
use crate::opt::SetOpt;
use crate::pm::Pm;
use std::str::FromStr;
use std::time::Duration;

// Represents a command from the GUI to the engine.
#[derive(Clone, Debug, PartialEq)]
pub enum GuiCmd {
    // uci: Tells the engine to switch to UCI mode.
    Uci,

    // debug: If true, then debug mode is enabled, otherwise it is disabled.
    Debug(bool),

    // isready: Used to synchronize the GUI with the engine. The command always
    // needs to be answered with readyok. If the engine is calculating, it
    // should also send readyok without stopping the calculation.
    IsReady,

    // setoption name <id> [value <x>]: This is sent to the engine when the user
    // wants to change the internal parameters of the engine. One command will
    // be sent for each parameter and this will only be sent when the engine is
    // waiting.
    SetOpt(SetOpt),

    // ucinewgame: Sent to the engine when the next search, started with
    // position and go will be from a different game.
    NewGame,

    // position [fen <fenstring> | startpos] moves <move1> ... <movei>: A
    // command to set up the initial position.
    Pos(Pos),

    // go [opts]: A command to tell the engine to begin calculating the best
    // move.
    Go(Go),

    // stop: A command to tell the engine to stop calculating as soon as
    // possible.
    Stop,

    // ponderhit: The user has played the expected move. This will be sent if
    // the engine was told to ponder on the same move the engine has played. The
    // engine has switched from pondering to normal search.
    Ponderhit,
}

impl FromStr for GuiCmd {
    type Err = UziErr;

    fn from_str(cmd: &str) -> Result<GuiCmd, Self::Err> {
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
            "setoption" => Ok(GuiCmd::SetOpt(SetOpt::try_from(&words)?)),
            "position" => Ok(GuiCmd::Pos(Pos::try_from(words.as_slice())?)),
            "go" => Ok(GuiCmd::Go(Go::try_from(words.as_slice())?)),
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
    // searchmoves <move1> ... <movei>: Restricts calculation by one or more
    // moves.
    search_moves: Option<Vec<Pm>>,

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

    // movestogo <x>: There are x moves to the next time control. If this is not
    // set, then wtime and btime represent sudden death.
    moves_to_go: Option<u16>,

    // depth <x>: Search x plies only.
    depth: Option<u16>,

    // nodes <x>: Search x nodes only.
    nodes: Option<u64>,

    // mate <x>: Search for a mate in x moves.
    mate: Option<u16>,

    // movetime <x>: Search exactly x milliseconds.
    move_time: Option<Duration>,

    // infinite: Search until the stop command. Do not exit search without being
    // told to do so in this mode.
    infinite: Option<()>,
}

impl Go {
    #[inline]
    pub fn new() -> Self {
        Go::default()
    }

    // Adds a search move to restrict search.
    pub fn add_search_move(&mut self, pm: Pm) -> &mut Self {
        if let Some(ref mut moves) = self.search_moves {
            moves.push(pm);
        } else {
            self.search_moves = Some(vec![pm]);
        }
        self
    }

    // Remaining time for white in seconds.
    pub fn set_wtime(&mut self, wtime: Duration) -> &mut Self {
        self.wtime.replace(wtime);
        self
    }

    // Remaining time for black in seconds.
    pub fn set_btime(&mut self, btime: Duration) -> &mut Self {
        self.btime.replace(btime);
        self
    }

    // Time increment for white in seconds.
    pub fn set_winc(&mut self, winc: Duration) -> &mut Self {
        self.winc.replace(winc);
        self
    }

    // Time increment for black in seconds.
    pub fn set_binc(&mut self, binc: Duration) -> &mut Self {
        self.binc.replace(binc);
        self
    }

    // Sets the number of moves until the next time control.
    pub fn set_moves_to_go(&mut self, moves_to_go: u16) -> &mut Self {
        self.moves_to_go.replace(moves_to_go);
        self
    }

    // Sets the depth limit in plies.
    pub fn set_depth(&mut self, depth: u16) -> &mut Self {
        self.depth.replace(depth);
        self
    }

    // Max total nodes to search.
    pub fn set_nodes(&mut self, nodes: u64) -> &mut Self {
        self.nodes.replace(nodes);
        self
    }

    // Moves to force mate.
    pub fn set_mate(&mut self, mate: u16) -> &mut Self {
        self.mate.replace(mate);
        self
    }

    // Number of seconds to spend for the move.
    pub fn set_move_time(&mut self, move_time: Duration) -> &mut Self {
        self.move_time.replace(move_time);
        self
    }

    // Search until the "stop" command is sent.
    pub fn set_infinite(&mut self) -> &mut Self {
        self.infinite.replace(());
        self
    }

    // Search in ponder mode.
    pub fn set_ponder(&mut self) -> &mut Self {
        self.ponder.replace(());
        self
    }

    // Returns true if any options are set.
    pub fn has_any(&self) -> bool {
        self.search_moves.is_some()
            || self.ponder.is_some()
            || self.wtime.is_some()
            || self.btime.is_some()
            || self.winc.is_some()
            || self.binc.is_some()
            || self.moves_to_go.is_some()
            || self.depth.is_some()
            || self.nodes.is_some()
            || self.mate.is_some()
            || self.move_time.is_some()
            || self.infinite.is_some()
    }
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

impl TryFrom<&[&str]> for Go {
    type Error = UziErr;

    fn try_from(cmd: &[&str]) -> Result<Go, Self::Error> {
        let mut go = Go::new();
        let mut parse_state = GoParseState::Begin;

        for word in cmd {
            match *word {
                "go" => {
                    if parse_state != GoParseState::Begin {
                        return Err(UziErr::GoErr);
                    }
                    parse_state = GoParseState::Go;
                }
                "wtime" => parse_state = GoParseState::Wtime,
                "btime" => parse_state = GoParseState::Btime,
                "winc" => parse_state = GoParseState::Winc,
                "binc" => parse_state = GoParseState::Binc,
                "movetime" => parse_state = GoParseState::MoveTime,
                "movestogo" => parse_state = GoParseState::MovesToGo,
                "depth" => parse_state = GoParseState::Depth,
                "nodes" => parse_state = GoParseState::Nodes,
                "mate" => parse_state = GoParseState::Mate,
                "searchmoves" => parse_state = GoParseState::SearchMoves,
                "infinite" => {
                    parse_state = GoParseState::Infinite;
                    go.set_infinite();
                }
                "ponder" => {
                    parse_state = GoParseState::Ponder;
                    go.set_ponder();
                }
                _ => parse_go_opt(parse_state, word, &mut go)?,
            }
        }

        if !go.has_any() {
            Err(UziErr::GoErr)
        } else {
            Ok(go)
        }
    }
}

// A helper function to parse and set the "go" command options.
fn parse_go_opt(parse_state: GoParseState, word: &str, go: &mut Go) -> Result<(), UziErr> {
    match parse_state {
        GoParseState::Wtime => go.set_wtime(to_millis(word, "wtime")?),
        GoParseState::Btime => go.set_btime(to_millis(word, "btime")?),
        GoParseState::Winc => go.set_winc(to_millis(word, "winc")?),
        GoParseState::Binc => go.set_binc(to_millis(word, "binc")?),
        GoParseState::MoveTime => go.set_move_time(to_millis(word, "movetime")?),
        GoParseState::MovesToGo => go.set_moves_to_go(to_number::<u16>(word)?),
        GoParseState::Depth => go.set_depth(to_number::<u16>(word)?),
        GoParseState::Nodes => go.set_nodes(to_number::<u64>(word)?),
        GoParseState::Mate => go.set_mate(to_number::<u16>(word)?),
        GoParseState::SearchMoves => go.add_search_move(Pm::from_str(word)?),
        _ => return Err(UziErr::GoErr),
    };
    Ok(())
}

// An enum to represent the current option being parsed when the go command is
// parsed.
#[derive(PartialEq, Clone, Copy)]
enum GoParseState {
    Begin,
    Go,
    SearchMoves,
    Ponder,
    Wtime,
    Btime,
    Winc,
    Binc,
    MovesToGo,
    Depth,
    Nodes,
    Mate,
    MoveTime,
    Infinite,
}

// A structure to represent the UCI "position" command, which is issued to the
// engine to set up the initial position, in the following format:
//
// position [fen <fenstring> | startpos] moves <move1> ... <movei>
//
// Set up the position described in fenstring or from the starting position and
// play the moves. No new command is needed, but if the position is from a
// different game than the last position sent to the engine, then the GUI should
// have sent a "ucinewgame" in between.
#[derive(Clone, Debug, PartialEq)]
pub struct Pos {
    // Represents the initial position: either a new game or a FEN string.
    pos: PosOpt,

    // Moves to apply to the initial position. If set, the intial position is
    // derived after the moves are applied to the initial position.
    moves: Option<Vec<Pm>>,
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
    pub fn add_move(&mut self, pm: Pm) -> &mut Self {
        if let Some(ref mut moves) = self.moves {
            moves.push(pm);
        } else {
            self.moves = Some(vec![pm]);
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

impl TryFrom<&[&str]> for Pos {
    type Error = UziErr;

    fn try_from(cmd: &[&str]) -> Result<Pos, Self::Error> {
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
                        pos.add_move(Pm::from_str(*word)?);
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
    use crate::opt::{Opponent, PlayerType, SetOpt, Title};

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
        let pm1 = Pm::from_str("e2e4").unwrap();
        let pm2 = Pm::from_str("e7e5").unwrap();
        let mut pos = Pos::new();
        pos.add_move(pm1).add_move(pm2);
        assert_eq!(pos.pos, PosOpt::StartPos);
        assert_eq!(pos.moves, Some(vec![pm1, pm2]));
    }

    #[test]
    fn pos_try_from_empty_vec() {
        let args = vec![];
        assert_eq!(Pos::try_from(args.as_slice()), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_with_only_position() {
        let args = ["position"];
        assert_eq!(Pos::try_from(&args[..]), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_pos_with_startpos() {
        let args = ["position", "startpos"];
        assert_eq!(Pos::try_from(&args[..]), Ok(Pos::new()));
    }

    #[test]
    fn pos_try_from_fen_but_missing_fen_string() {
        let args = ["position", "fen"];
        assert_eq!(Pos::try_from(&args[..]), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_with_ranom_string() {
        let args = ["position", "random", "fen"];
        assert_eq!(Pos::try_from(&args[..]), Err(UziErr::Position));
    }

    #[test]
    fn pos_try_from_fen_with_fen() {
        let args = ["position", "fen", "FENSTRING"];
        assert_eq!(Pos::try_from(&args[..]), Ok(Pos::with_fen("FENSTRING")));
    }

    #[test]
    fn pos_try_from_fen_with_moves() {
        let args = ["position", "fen", "FENSTRING", "moves", "e2e4", "e7e5"];
        let mut pos = Pos::with_fen("FENSTRING");
        pos.add_move(Pm::from_str("e2e4").unwrap())
            .add_move(Pm::from_str("e7e5").unwrap());
        assert_eq!(Pos::try_from(&args[..]), Ok(pos));
    }

    #[test]
    fn pos_try_from_startpos_with_moves() {
        let args = ["position", "startpos", "moves", "e2e4", "e7e5"];
        let mut pos = Pos::new();
        pos.add_move(Pm::from_str("e2e4").unwrap())
            .add_move(Pm::from_str("e7e5").unwrap());
        assert_eq!(Pos::try_from(&args[..]), Ok(pos));
    }

    #[test]
    fn pos_try_from_mixed_fen_and_startpos() {
        let args = ["position", "startpos", "fen"];
        assert_eq!(Pos::try_from(&args[..]), Err(UziErr::Position));
    }

    #[test]
    fn go_default_without_any_opts() {
        let go = Go::new();
        assert!(!go.has_any());
    }

    #[test]
    fn go_with_all_opts() {
        let mut go = Go::new();
        go.add_search_move(Pm::from_str("e2e4").unwrap())
            .set_ponder()
            .set_wtime(Duration::from_millis(1))
            .set_btime(Duration::from_millis(2))
            .set_winc(Duration::from_millis(1))
            .set_binc(Duration::from_millis(2))
            .set_moves_to_go(10)
            .set_depth(100)
            .set_nodes(100_000)
            .set_mate(10)
            .set_move_time(Duration::from_millis(100))
            .set_infinite();
        assert_eq!(
            go,
            Go {
                search_moves: Some(vec![Pm::from_str("e2e4").unwrap()]),
                ponder: Some(()),
                wtime: Some(Duration::from_millis(1)),
                btime: Some(Duration::from_millis(2)),
                winc: Some(Duration::from_millis(1)),
                binc: Some(Duration::from_millis(2)),
                moves_to_go: Some(10),
                depth: Some(100),
                nodes: Some(100_000),
                mate: Some(10),
                move_time: Some(Duration::from_millis(100)),
                infinite: Some(()),
            }
        );
    }

    #[test]
    fn go_try_from_empty() {
        assert_eq!(Go::try_from(&["hello", "mother"][..]), Err(UziErr::GoErr));
    }

    #[test]
    fn go_try_from_all_opts() {
        let opts = [
            "go",
            "searchmoves",
            "e2e4",
            "e7e5",
            "ponder",
            "wtime",
            "1",
            "btime",
            "2",
            "winc",
            "1",
            "binc",
            "2",
            "movestogo",
            "10",
            "depth",
            "100",
            "nodes",
            "100000",
            "mate",
            "10",
            "movetime",
            "100",
            "infinite",
        ];
        assert_eq!(
            Go::try_from(&opts[..]),
            Ok(Go {
                search_moves: Some(vec![
                    Pm::from_str("e2e4").unwrap(),
                    Pm::from_str("e7e5").unwrap()
                ]),
                ponder: Some(()),
                wtime: Some(Duration::from_millis(1)),
                btime: Some(Duration::from_millis(2)),
                winc: Some(Duration::from_millis(1)),
                binc: Some(Duration::from_millis(2)),
                moves_to_go: Some(10),
                depth: Some(100),
                nodes: Some(100_000),
                mate: Some(10),
                move_time: Some(Duration::from_millis(100)),
                infinite: Some(()),
            })
        );
    }

    #[test]
    fn guicmd_simple() {
        assert_eq!(GuiCmd::from_str("uci"), Ok(GuiCmd::Uci));
        assert_eq!(GuiCmd::from_str("isready"), Ok(GuiCmd::IsReady));
        assert_eq!(GuiCmd::from_str("ucinewgame"), Ok(GuiCmd::NewGame));
        assert_eq!(GuiCmd::from_str("stop"), Ok(GuiCmd::Stop));
        assert_eq!(GuiCmd::from_str("ponderhit"), Ok(GuiCmd::Ponderhit));
        assert_eq!(GuiCmd::from_str("debug on"), Ok(GuiCmd::Debug(true)));
        assert_eq!(GuiCmd::from_str("debug off"), Ok(GuiCmd::Debug(false)));
        assert_eq!(GuiCmd::from_str("hello"), Err(UziErr::What));
    }

    #[test]
    fn guicmd_setopt() {
        assert_eq!(
            GuiCmd::from_str("setoption name UCI_Opponent value IM 2300 human oserr"),
            Ok(GuiCmd::SetOpt(SetOpt::Opp(Opponent {
                title: Title::IM,
                elo: Some(2300),
                player_type: PlayerType::Human,
                name: "oserr".into()
            })))
        );
    }

    #[test]
    fn guicmd_position() {
        assert_eq!(
            GuiCmd::from_str("position fen FENSTRING moves e2e4 e7e5"),
            Ok(GuiCmd::Pos(Pos {
                pos: PosOpt::Fen("FENSTRING".into()),
                moves: Some(vec![
                    Pm::from_str("e2e4").unwrap(),
                    Pm::from_str("e7e5").unwrap()
                ]),
            }))
        );
    }

    #[test]
    fn guicmd_go() {
        let mut go = Go::new();
        go.add_search_move(Pm::from_str("e2e4").unwrap())
            .add_search_move(Pm::from_str("e7e5").unwrap())
            .set_wtime(Duration::from_millis(1))
            .set_btime(Duration::from_millis(2));
        assert_eq!(
            GuiCmd::from_str("go searchmoves e2e4 e7e5 wtime 1 btime 2"),
            Ok(GuiCmd::Go(go))
        );
    }
}

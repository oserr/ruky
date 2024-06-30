// This module contains the types to represent a UCI option.

use crate::conv::{to_bool, to_number};
use crate::err::UziErr;
use std::path::PathBuf;
use std::str::FromStr;

// Represents all the different options that may be supported by a UCI compliant
// chess engine.
#[derive(Clone, Debug, PartialEq)]
pub enum UciOpt {
    // The value in MB for memory for hash tables.
    Hash,
    // The path on the hard disk to the Nalimov compressed format. Multiple directories can be
    // concatenated with ";".
    NalimovPath(PathBuf),
    // This is the size in MB for the cache for the nalimov table bases.
    NalimovCache(OptVal),
    // This means that the engine is able to ponder.
    Ponder(bool),
    // This means that the engine has its own book which is accessed by the engine itself. If this
    // is set, the engine takes care of the opening book. If set to false, the engine should not
    // its book.
    OwnBook(bool),
    // The engine supports multi best line or k-best mode. The default value is 1.
    MultiPv(OptVal),
    // UCI_ShowCurrLine: The engine can show the current line it is calculating.
    ShowCurrLine(bool),
    // UCI_ShowRefutations: The engine can show a move and its refutations in a line.
    ShowRefutations(bool),
    // UCI_LimitStrength: The engine is able to limit its strength to a specific elo rating. This
    // should always be implemented together with "UCI_Elo".
    LimitStrength(bool),
    // UCI_Elo: The engine can limit its strengh in Elo within this interval. Should be
    // implemented together with UCI_LimitStrength.
    Elo(OptVal),
    // UCI_AnalsysMode: The engine wants to behave differently when analysing or playing a game.
    // This is set to false if the engine is playing a game.
    AnalysisMode(bool),
    // UCI_Opponent: The command can be used by the GUI to send the name, title, elo and if the
    // engine is playing a human or computer to the engine. The format of the string is:
    // - [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>, e.g.:
    // - setoption name UCI_Opponent value GM 2800 human Garry Kasparov
    // - setoption name UCI_Opponent value none none computer Shredder
    Opponent {
        title: Option<Title>,
        elo: Option<u32>,
        player_type: PlayerType,
        name: String,
    },
    // UCI_EngineAbout: The engine tells the GUI information about itself.
    About(String),
    // UCI_ShredderbasesPath: Path to folder of containing the Shredder endgame databases.
    ShredderBasesPath(PathBuf),
    // UCI_SetPositionValue: The GUI can send this to the engine to tell it to use a certain value
    // in centipawns from white's point of view if evaluating this specific position. Allowed
    // formats:
    SetPositionValue,
    // type <t>: The option has type <t>: check, spin, combo, button, string.
    Type(OptType),
}

// The type of an option.
#[derive(Clone, Debug, PartialEq)]
pub enum OptType {
    Check,
    Spin,
    Combo,
    Button,
    Str,
}

// Represents either the value an option can take, or the value to set an
// option.
#[derive(Clone, Debug, PartialEq)]
pub enum OptVal {
    // Used to set a value, from GUI to engine.
    Set(i64),

    // Used to tell the GUI about the values an option can take.
    Info {
        default: i64,
        min: i64,
        max: i64,
        var: Option<i64>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SetOpt {
    // The value in MB for memory for hash tables.
    Hash(u64),
    // The path on the hard disk to the Nalimov compressed format. Multiple directories can be
    // concatenated with ";".
    NalimovPath(PathBuf),
    // This is the size in MB for the cache for the nalimov table bases.
    NalimovCache(u64),
    // If set, the engine can think of the next move while the opponent is thinking.
    Ponder(bool),
    // This means that the engine has its own book which is accessed by the engine itself. If this
    // is set, the engine takes care of the opening book. If set to false, the engine should not
    // its book.
    OwnBook(bool),
    // The engine supports multi best line or k-best mode. The default value is 1.
    MultiPv(u64),
    // UCI_ShowCurrLine: The engine can show the current line it is calculating.
    ShowCurrLine(bool),
    // UCI_ShowRefutations: The engine can show a move and its refutations in a line.
    ShowRefutations(bool),
    // UCI_LimitStrength: The engine is able to limit its strength to a specific elo rating. This
    // should always be implemented together with "UCI_Elo".
    LimitStrength(bool),
    // UCI_Elo: The engine can limit its strengh in Elo within this interval. Should be
    // implemented together with UCI_LimitStrength.
    Elo(u16),
    // UCI_AnalsysMode: The engine wants to behave differently when analysing or playing a game.
    // This is set to false if the engine is playing a game.
    AnalysisMode(bool),
    // UCI_ShredderbasesPath: Path to folder of containing the Shredder endgame databases.
    ShredderBasesPath(PathBuf),
    // UCI_Opponent: Used to set the opponent info.
    Opp(Opponent),
    // UCI_SetPositionValue: The GUI can send this to the engine to tell it to use a certain value
    // in centipawns from white's point of view if evaluating this specific position. Allowed
    // formats:
    SetPositionValue(PosValueOpt),
}

impl TryFrom<&Vec<&str>> for SetOpt {
    type Error = UziErr;
    fn try_from(cmd: &Vec<&str>) -> Result<Self, Self::Error> {
        let mut parse_state = SetOptParseState::Begin;
        for (i, word) in cmd.into_iter().enumerate() {
            match *word {
                "setoption" if parse_state.is_begin() => parse_state = SetOptParseState::SetOpt,
                "name" if parse_state.is_setopt() => parse_state = SetOptParseState::Name,
                "value" if parse_state.is_val() => {
                    if let Some(opt) = parse_state.get_val() {
                        return parse_value(opt, &cmd[i..]);
                    } else {
                        return Err(UziErr::SetOptErr);
                    }
                }
                _ => match parse_state {
                    SetOptParseState::Name => {
                        let opt = UziOpt::from_str(*word)?;
                        parse_state = SetOptParseState::Value(opt);
                    }
                    _ => return Err(UziErr::SetOptErr),
                },
            }
        }
        Err(UziErr::SetOptErr)
    }
}

// Parse the value in the "setoption" command and creates a SetOpt if there is
// no error, otherwise returns an error.
fn parse_value(opt: UziOpt, cmd: &[&str]) -> Result<SetOpt, UziErr> {
    if cmd.is_empty() {
        return Err(UziErr::SetOptErr);
    }
    let word = cmd[0];
    match opt {
        UziOpt::About => Err(UziErr::SetOptErr),
        UziOpt::Hash => Ok(SetOpt::Hash(to_number::<u64>(word)?)),
        UziOpt::NalimovPath => Ok(SetOpt::NalimovPath(PathBuf::from_str(word).unwrap())),
        UziOpt::NalimovCache => Ok(SetOpt::NalimovCache(to_number::<u64>(word)?)),
        UziOpt::Ponder => Ok(SetOpt::Ponder(to_bool(word)?)),
        UziOpt::OwnBook => Ok(SetOpt::OwnBook(to_bool(word)?)),
        UziOpt::MultiPv => Ok(SetOpt::MultiPv(to_number::<u64>(word)?)),
        UziOpt::ShowCurrLine => Ok(SetOpt::ShowCurrLine(to_bool(word)?)),
        UziOpt::ShowRefutations => Ok(SetOpt::ShowRefutations(to_bool(word)?)),
        UziOpt::LimitStrength => Ok(SetOpt::LimitStrength(to_bool(word)?)),
        UziOpt::Elo => Ok(SetOpt::Elo(to_number::<u16>(word)?)),
        UziOpt::AnalysisMode => Ok(SetOpt::AnalysisMode(to_bool(word)?)),
        UziOpt::ShredderBasesPath => {
            Ok(SetOpt::ShredderBasesPath(PathBuf::from_str(word).unwrap()))
        }
        UziOpt::Opponent => Ok(SetOpt::Opp(Opponent::try_from(cmd)?)),
        UziOpt::SetPositionValue => parse_position_val(&cmd),
    }
}

#[derive(PartialEq)]
enum SetOptParseState {
    Begin,
    SetOpt,
    Name,
    Value(UziOpt),
}

impl SetOptParseState {
    #[inline]
    fn is_begin(&self) -> bool {
        matches!(*self, SetOptParseState::Begin)
    }

    #[inline]
    fn is_setopt(&self) -> bool {
        matches!(*self, SetOptParseState::SetOpt)
    }

    #[inline]
    fn is_val(&self) -> bool {
        matches!(*self, SetOptParseState::Value(_))
    }

    fn get_val(&self) -> Option<UziOpt> {
        match *self {
            SetOptParseState::Value(opt) => Some(opt),
            _ => None,
        }
    }
}

// Represents all the UCI options, but we don't use payloads here.
#[derive(Copy, Clone, PartialEq)]
pub enum UziOpt {
    Hash,
    NalimovPath,
    NalimovCache,
    Ponder,
    OwnBook,
    MultiPv,
    ShowCurrLine,
    ShowRefutations,
    LimitStrength,
    Elo,
    AnalysisMode,
    Opponent,
    About,
    ShredderBasesPath,
    SetPositionValue,
}

impl FromStr for UziOpt {
    type Err = UziErr;

    fn from_str(buf: &str) -> Result<Self, Self::Err> {
        match buf {
            HASH => Ok(UziOpt::Hash),
            NALIMOV_PATH => Ok(UziOpt::NalimovPath),
            NALIMOVE_CACHE => Ok(UziOpt::NalimovCache),
            PONDER => Ok(UziOpt::Ponder),
            OWN_BOOK => Ok(UziOpt::OwnBook),
            MULTI_PV => Ok(UziOpt::MultiPv),
            SHOW_CURR_LINE => Ok(UziOpt::ShowCurrLine),
            SHOW_REFUTATIONS => Ok(UziOpt::ShowRefutations),
            LIMIT_STRENGTH => Ok(UziOpt::LimitStrength),
            ELO => Ok(UziOpt::Elo),
            ANALYSIS_MODE => Ok(UziOpt::AnalysisMode),
            OPPONENT => Ok(UziOpt::Opponent),
            SHREDDER_BASES_PATH => Ok(UziOpt::ShredderBasesPath),
            SET_POSITION_VALUE => Ok(UziOpt::SetPositionValue),
            _ => Err(UziErr::UnknownOpt),
        }
    }
}

// Represents the opponent option: UCI_Opponent.
// The command can be used by the GUI to send the name, title, elo and if the
// engine is playing a human or computer to the engine. The format of the string
// is:
// - [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>, e.g.:
// - setoption name UCI_Opponent value GM 2800 human Garry Kasparov
// - setoption name UCI_Opponent value none none computer Shredder
#[derive(Clone, Debug, PartialEq)]
pub struct Opponent {
    title: Title,
    elo: Option<u16>,
    player_type: PlayerType,
    name: String,
}

impl Default for Opponent {
    fn default() -> Self {
        Self {
            title: Title::NoTitle,
            elo: None,
            player_type: PlayerType::Human,
            name: String::new(),
        }
    }
}

impl TryFrom<&[&str]> for Opponent {
    type Error = UziErr;

    fn try_from(opts: &[&str]) -> Result<Opponent, UziErr> {
        if opts.len() != 4 {
            return Err(UziErr::BadOpponent);
        }

        let mut opp = Opponent::default();

        for (i, word) in opts.into_iter().enumerate() {
            match i {
                0 => opp.title = Title::from_str(word)?,
                1 if *word == "none" => continue,
                1 => {
                    let elo = to_number::<u16>(word).map_err(|_| UziErr::BadOpponent)?;
                    opp.elo = Some(elo);
                }
                2 => opp.player_type = PlayerType::from_str(word)?,
                3 => opp.name.push_str(word),
                _ => return Err(UziErr::BadOpponent),
            }
        }

        Ok(opp)
    }
}

// Represents the title of the player, e.g. grand master.
#[derive(Clone, Debug, PartialEq)]
pub enum Title {
    GM,
    IM,
    FM,
    WGM,
    WIM,
    NoTitle,
}

impl FromStr for Title {
    type Err = UziErr;

    fn from_str(buf: &str) -> Result<Title, UziErr> {
        match buf {
            "GM" => Ok(Title::GM),
            "IM" => Ok(Title::IM),
            "FM" => Ok(Title::FM),
            "WGM" => Ok(Title::WGM),
            "WIM" => Ok(Title::WIM),
            "none" => Ok(Title::NoTitle),
            _ => Err(UziErr::BadTitle),
        }
    }
}

// To represent human or computer players.
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerType {
    Human,
    Computer,
}

impl FromStr for PlayerType {
    type Err = UziErr;

    fn from_str(buf: &str) -> Result<Self, Self::Err> {
        match buf {
            "human" => Ok(PlayerType::Human),
            "computer" => Ok(PlayerType::Computer),
            _ => Err(UziErr::BadPlayerType),
        }
    }
}

// Represents the different values that can be used for UCI_SetPositionValue.
#[derive(Clone, Debug, PartialEq)]
pub enum PosValueOpt {
    // <value> + <fen>
    Val { val: i32, fen: String },
    // clear + <fen>
    Clear(String),
    // clearall
    ClearAll,
}

impl TryFrom<&[&str]> for PosValueOpt {
    type Error = UziErr;

    fn try_from(opts: &[&str]) -> Result<Self, Self::Error> {
        if opts.len() == 1 && opts[0] == "clearall" {
            return Ok(PosValueOpt::ClearAll);
        }

        if opts.len() != 2 {
            return Err(UziErr::BadPositionVal);
        }

        match opts[0] {
            "clear" => Ok(PosValueOpt::Clear(opts[1].into())),
            _ => Ok(PosValueOpt::Val {
                val: to_number::<i32>(opts[0]).map_err(|_| UziErr::BadPositionVal)?,
                fen: opts[1].into(),
            }),
        }
    }
}

fn parse_position_val(cmd: &[&str]) -> Result<SetOpt, UziErr> {
    todo!()
}

// Constants for the names of the UCI options.
const HASH: &str = "Hash";
const NALIMOV_PATH: &str = "NalimovPath";
const NALIMOVE_CACHE: &str = "NalimovCache";
const OWN_BOOK: &str = "OwnBook";
const MULTI_PV: &str = "MultiPv";
const PONDER: &str = "Ponder";
const SHOW_CURR_LINE: &str = &"UCI_ShowCurrLine";
const SHOW_REFUTATIONS: &str = "UCI_ShowRefutations";
const LIMIT_STRENGTH: &str = "UCI_LimitStrength";
const ELO: &str = "UCI_Elo";
const ANALYSIS_MODE: &str = "UCI_AnalysisMode";
const OPPONENT: &str = "UCI_Opponent";
const SHREDDER_BASES_PATH: &str = "UCI_ShredderbasesPath";
const SET_POSITION_VALUE: &str = "UCI_SetPositionValue";

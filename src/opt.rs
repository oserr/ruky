// This module contains the types to represent a UCI option.

use crate::conv::{to_bool, to_number};
use crate::err::UziErr;
use crate::types::{CheckType, SpinType, StrType};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

// Represents all the different options that may be supported by a UCI compliant
// chess engine. These are meant to be used by the engine to tell the GUI which
// options are available, and what their default configurations are.
// TODO: Add support for non-standard options.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HasOpt {
    // The value in MB for memory for hash tables.
    Hash(SpinType<u64>),
    // The path on the hard disk to the Nalimov compressed format.
    NalimovPath(StrType),
    // This is the size in MB for the cache for the nalimov table bases.
    NalimovCache(SpinType<u64>),
    // Tells the GUI the engine is able to ponder.
    Ponder(CheckType),
    // Tells the GUI the engine has its own opening book.
    OwnBook(CheckType),
    // Tells teh GUI the engine supports k-best mode.
    MultiPv(SpinType<u64>),
    // UCI_ShowCurrLine: Tells the GUI the engine can show the current line it
    // is calculating.
    ShowCurrLine(CheckType),
    // UCI_ShowRefutations: Tells the GUI the engine can show a move and its
    // refutations in a line.
    ShowRefutations(CheckType),
    // UCI_LimitStrength: Tells the GUI that the engine can limit its strength.
    LimitStrength(CheckType),
    // UCI_Elo: Tells the GUI that it can limit its strength within an Elo
    // interval.
    Elo(SpinType<u16>),
    // UCI_AnalsysMode: Tells the GUI that the engine supports analysis mode, in
    // which case behavior might be different than when it is playing a game.
    AnalysisMode(CheckType),
    // UCI_ShredderbasesPath: Tells the GUI that the engine supports the
    // Shredder end-game databa.
    ShredderBasesPath(StrType),
    // UCI_Opponent: Tells the GUI how to configure the Opponent.
    Opp(StrType),
    // UCI_SetPositionValue: Tells the GUI that the engine supports evaluating
    // positions with values provided by the GUI.
    SetPosVal(StrType),
    // UCI_EngineAbout: Tells the GUI about the engine.
    About(StrType),
}

impl Display for HasOpt {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "option name ")?;
        match self {
            HasOpt::Hash(t) => write!(formatter, "{} {}", HASH, t),
            HasOpt::NalimovPath(t) => write!(formatter, "{} {}", NALIMOV_PATH, t),
            HasOpt::NalimovCache(t) => write!(formatter, "{} {}", NALIMOV_CACHE, t),
            HasOpt::Ponder(t) => write!(formatter, "{} {}", PONDER, t),
            HasOpt::OwnBook(t) => write!(formatter, "{} {}", OWN_BOOK, t),
            HasOpt::MultiPv(t) => write!(formatter, "{} {}", MULTI_PV, t),
            HasOpt::ShowCurrLine(t) => write!(formatter, "{} {}", SHOW_CURR_LINE, t),
            HasOpt::ShowRefutations(t) => write!(formatter, "{} {}", SHOW_REFUTATIONS, t),
            HasOpt::LimitStrength(t) => write!(formatter, "{} {}", LIMIT_STRENGTH, t),
            HasOpt::Elo(t) => write!(formatter, "{} {}", ELO, t),
            HasOpt::AnalysisMode(t) => write!(formatter, "{} {}", ANALYSIS_MODE, t),
            HasOpt::ShredderBasesPath(t) => write!(formatter, "{} {}", SHREDDER_BASES_PATH, t),
            HasOpt::Opp(t) => write!(formatter, "{} {}", OPPONENT, t),
            HasOpt::SetPosVal(t) => write!(formatter, "{} {}", SET_POSITION_VALUE, t),
            HasOpt::About(t) => write!(formatter, "{} {}", ABOUT, t),
        }
    }
}

// TODO: Add support for non-standard options.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SetOpt {
    // The value in MB for memory for hash tables.
    Hash(u64),
    // The path on the hard disk to the Nalimov compressed format. Multiple
    // directories can be concatenated with ";".
    NalimovPath(PathBuf),
    // This is the size in MB for the cache for the nalimov table bases.
    NalimovCache(u64),
    // If set, the engine can think of the next move while the opponent is
    // thinking.
    Ponder(bool),
    // If set, enables the engine to use its own opening book.
    OwnBook(bool),
    // Used by the GUI to tell the engine to use k-best mode. The default value
    // is 1.
    MultiPv(u64),
    // UCI_ShowCurrLine: If set to true, the engine shows the current line it is
    // calculating.
    ShowCurrLine(bool),
    // UCI_ShowRefutations: If set to true, the engine can show a move and its
    // refutations in a line.
    ShowRefutations(bool),
    // UCI_LimitStrength: If set, the engine limits its strength to a specific
    // ELO rating. If this is enabled, the GUI must alsot set "UCI_Elo" to an
    // appropriate ELO score.
    LimitStrength(bool),
    // UCI_Elo: If set, the engine plays at a strength that is accordance with
    // the Elo rating. Must be set in tandem with the "UCI_LimitStrength".
    Elo(u16),
    // UCI_AnalsysMode: Tells the engine to use analysis mode.
    AnalysisMode(bool),
    // UCI_ShredderbasesPath: Sets the path to a folder containing the Shredder
    // endgame databases.
    ShredderBasesPath(PathBuf),
    // UCI_Opponent: Tells the engine the name, title, elo and if the engine is
    // playing a human or computer to the engine. The format of the string is:
    // - [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>, e.g.:
    // - setoption name UCI_Opponent value GM 2800 human Garry Kasparov
    // - setoption name UCI_Opponent value none none computer Shredder
    Opp(Opponent),
    // UCI_SetPositionValue: Tells the engine to use a certain value in
    // centipawns from white's point of view if evaluating this specific
    // position. See PosValueOpt for accepted formats.
    SetPosVal(PosValueOpt),
}

impl TryFrom<&[&str]> for SetOpt {
    type Error = UziErr;
    fn try_from(cmd: &[&str]) -> Result<Self, Self::Error> {
        let mut parse_state = SetOptParseState::Begin;
        for (i, word) in cmd.into_iter().enumerate() {
            match *word {
                "setoption" if parse_state.is_begin() => parse_state = SetOptParseState::SetOpt,
                "name" if parse_state.is_setopt() => parse_state = SetOptParseState::Name,
                "value" if parse_state.is_val() => continue,
                _ => match parse_state {
                    SetOptParseState::Name => {
                        let opt = UziOpt::from_str(*word)?;
                        parse_state = SetOptParseState::Value(opt);
                    }
                    SetOptParseState::Value(opt) => return parse_value(opt, &cmd[i..]),
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
        UziOpt::SetPosVal => Ok(SetOpt::SetPosVal(PosValueOpt::try_from(cmd)?)),
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
}

// Represents all the UCI options, but we don't use payloads here.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    SetPosVal,
}

impl FromStr for UziOpt {
    type Err = UziErr;

    fn from_str(buf: &str) -> Result<Self, Self::Err> {
        match buf {
            ABOUT => Ok(UziOpt::About),
            HASH => Ok(UziOpt::Hash),
            NALIMOV_PATH => Ok(UziOpt::NalimovPath),
            NALIMOV_CACHE => Ok(UziOpt::NalimovCache),
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
            SET_POSITION_VALUE => Ok(UziOpt::SetPosVal),
            _ => Err(UziErr::UnknownOpt),
        }
    }
}

impl IntoIterator for UziOpt {
    type Item = UziOpt;
    type IntoIter = UziOptIter;
    fn into_iter(self) -> Self::IntoIter {
        UziOptIter { curr: Some(self) }
    }
}

pub struct UziOptIter {
    curr: Option<UziOpt>,
}

impl Iterator for UziOptIter {
    type Item = UziOpt;
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.curr;
        self.curr = match self.curr {
            None => None,
            Some(opt) => match opt {
                UziOpt::Hash => Some(UziOpt::NalimovPath),
                UziOpt::NalimovPath => Some(UziOpt::NalimovCache),
                UziOpt::NalimovCache => Some(UziOpt::Ponder),
                UziOpt::Ponder => Some(UziOpt::OwnBook),
                UziOpt::OwnBook => Some(UziOpt::MultiPv),
                UziOpt::MultiPv => Some(UziOpt::ShowCurrLine),
                UziOpt::ShowCurrLine => Some(UziOpt::ShowRefutations),
                UziOpt::ShowRefutations => Some(UziOpt::LimitStrength),
                UziOpt::LimitStrength => Some(UziOpt::Elo),
                UziOpt::Elo => Some(UziOpt::AnalysisMode),
                UziOpt::AnalysisMode => Some(UziOpt::Opponent),
                UziOpt::Opponent => Some(UziOpt::About),
                UziOpt::About => Some(UziOpt::ShredderBasesPath),
                UziOpt::ShredderBasesPath => Some(UziOpt::SetPosVal),
                UziOpt::SetPosVal => None,
            },
        };
        val
    }
}

// Represents the opponent option: UCI_Opponent.
// The command can be used by the GUI to send the name, title, elo and if the
// engine is playing a human or computer to the engine. The format of the string
// is:
// - [GM|IM|FM|WGM|WIM|none] [<elo>|none] [computer|human] <name>, e.g.:
// - setoption name UCI_Opponent value GM 2800 human Garry Kasparov
// - setoption name UCI_Opponent value none none computer Shredder
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Opponent {
    pub title: Title,
    pub elo: Option<u16>,
    pub player_type: PlayerType,
    pub name: String,
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

impl Display for Opponent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let elo = self
            .elo
            .map_or_else(|| "none".to_string(), |x| x.to_string());
        write!(
            formatter,
            "{} {} {} {}",
            self.title, elo, self.player_type, self.name
        )
    }
}

impl From<Opponent> for StrType {
    fn from(opp: Opponent) -> Self {
        StrType(opp.to_string())
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Title {
    GM,
    IM,
    FM,
    WGM,
    WIM,
    NoTitle,
}

impl Title {
    pub fn as_str(&self) -> &'static str {
        match self {
            Title::GM => "GM",
            Title::IM => "IM",
            Title::FM => "FM",
            Title::WGM => "WGM",
            Title::WIM => "WIM",
            Title::NoTitle => "none",
        }
    }
}

impl Display for Title {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlayerType {
    Human,
    Computer,
}

impl PlayerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlayerType::Human => "human",
            PlayerType::Computer => "computer",
        }
    }
}

impl Display for PlayerType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

        if opts.len() < 2 {
            return Err(UziErr::BadPositionVal);
        }

        let fen = opts[1..].join(" ");

        match opts[0] {
            "clear" => Ok(PosValueOpt::Clear(fen)),
            _ => Ok(PosValueOpt::Val {
                val: to_number::<i32>(opts[0]).map_err(|_| UziErr::BadPositionVal)?,
                fen: fen,
            }),
        }
    }
}

impl Display for PosValueOpt {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PosValueOpt::Val { val, fen } => write!(formatter, "{} {}", val, fen),
            PosValueOpt::Clear(fen) => write!(formatter, "clear {}", fen),
            PosValueOpt::ClearAll => formatter.write_str("clearall"),
        }
    }
}

impl From<PosValueOpt> for StrType {
    fn from(pos_val: PosValueOpt) -> Self {
        Self(pos_val.to_string())
    }
}

// Constants for the names of the UCI options.
const HASH: &str = "Hash";
const NALIMOV_PATH: &str = "NalimovPath";
const NALIMOV_CACHE: &str = "NalimovCache";
const OWN_BOOK: &str = "OwnBook";
const MULTI_PV: &str = "MultiPv";
const PONDER: &str = "Ponder";
const ABOUT: &str = "UCI_EngineAbout";
const SHOW_CURR_LINE: &str = &"UCI_ShowCurrLine";
const SHOW_REFUTATIONS: &str = "UCI_ShowRefutations";
const LIMIT_STRENGTH: &str = "UCI_LimitStrength";
const ELO: &str = "UCI_Elo";
const ANALYSIS_MODE: &str = "UCI_AnalysisMode";
const OPPONENT: &str = "UCI_Opponent";
const SHREDDER_BASES_PATH: &str = "UCI_ShredderbasesPath";
const SET_POSITION_VALUE: &str = "UCI_SetPositionValue";

#[cfg(test)]
mod tests {
    use super::*;

    const FEN_STR: &str = "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 40 50";

    #[test]
    fn try_from_returns_err_missing_options() {
        let mut opts = Vec::new();
        assert_eq!(SetOpt::try_from(opts.as_slice()), Err(UziErr::SetOptErr));
        opts.push("setoption");
        assert_eq!(SetOpt::try_from(opts.as_slice()), Err(UziErr::SetOptErr));
        opts.push("name");
        assert_eq!(SetOpt::try_from(opts.as_slice()), Err(UziErr::SetOptErr));
    }

    #[test]
    fn set_opt_try_from_returns_err_for_unknown_opt() {
        let opts = ["setoption", "name", "CheeseBurger"];
        assert_eq!(SetOpt::try_from(&opts[..]), Err(UziErr::UnknownOpt));
    }

    #[test]
    fn set_opt_try_from_returns_err_for_missing_val() {
        let opts = ["setoption", "name", HASH];
        assert_eq!(SetOpt::try_from(&opts[..]), Err(UziErr::SetOptErr));
    }

    #[test]
    fn set_opt_try_from_hash() {
        let opts = ["setoption", "name", HASH, "value", "128"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::Hash(128)));
    }

    #[test]
    fn set_opt_try_from_nalimov_path() {
        let opts = ["setoption", "name", NALIMOV_PATH, "value", "some/path"];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::NalimovPath(PathBuf::from_str("some/path").unwrap()))
        );
    }

    #[test]
    fn set_opt_try_from_nalimov_cache() {
        let opts = ["setoption", "name", NALIMOV_CACHE, "value", "256000"];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::NalimovCache(256_000))
        );
    }

    #[test]
    fn set_opt_try_from_ponder() {
        let opts = ["setoption", "name", PONDER, "value", "true"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::Ponder(true)));
        let opts = ["setoption", "name", PONDER, "value", "false"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::Ponder(false)));
    }

    #[test]
    fn set_opt_try_own_book() {
        let opts = ["setoption", "name", OWN_BOOK, "value", "true"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::OwnBook(true)));
        let opts = ["setoption", "name", OWN_BOOK, "value", "false"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::OwnBook(false)));
    }

    #[test]
    fn set_opt_try_multipv() {
        let opts = ["setoption", "name", MULTI_PV, "value", "16"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::MultiPv(16)));
    }

    #[test]
    fn set_opt_try_show_curr_line() {
        let opts = ["setoption", "name", SHOW_CURR_LINE, "value", "false"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::ShowCurrLine(false)));
        let opts = ["setoption", "name", SHOW_CURR_LINE, "value", "true"];
        assert_eq!(SetOpt::try_from(&opts[..]), Ok(SetOpt::ShowCurrLine(true)));
    }

    #[test]
    fn set_opt_try_from_opponent() {
        let opts = ["setoption", "name", OPPONENT, "value", "none"];
        assert_eq!(SetOpt::try_from(&opts[..]), Err(UziErr::BadOpponent));

        let opts = [
            "setoption",
            "name",
            OPPONENT,
            "value",
            "none",
            "none",
            "human",
            "oserr",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::Opp(Opponent {
                title: Title::NoTitle,
                elo: None,
                player_type: PlayerType::Human,
                name: "oserr".into()
            }))
        );

        let opts = [
            "setoption",
            "name",
            OPPONENT,
            "value",
            "none",
            "2200",
            "computer",
            "oserr",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::Opp(Opponent {
                title: Title::NoTitle,
                elo: Some(2200),
                player_type: PlayerType::Computer,
                name: "oserr".into()
            }))
        );

        let opts = [
            "setoption",
            "name",
            OPPONENT,
            "value",
            "GM",
            "4800",
            "computer",
            "ruky",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::Opp(Opponent {
                title: Title::GM,
                elo: Some(4800),
                player_type: PlayerType::Computer,
                name: "ruky".into()
            }))
        );

        let opts = [
            "setoption",
            "name",
            OPPONENT,
            "value",
            "zzz",
            "4800",
            "computer",
            "ruky",
        ];
        assert_eq!(SetOpt::try_from(&opts[..]), Err(UziErr::BadTitle));
    }

    #[test]
    fn set_opt_try_from_pos_value() {
        let opts = ["setoption", "name", SET_POSITION_VALUE, "value", "clearall"];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::SetPosVal(PosValueOpt::ClearAll))
        );

        let opts = [
            "setoption",
            "name",
            SET_POSITION_VALUE,
            "value",
            "clear",
            "fen",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::SetPosVal(PosValueOpt::Clear("fen".into())))
        );

        let opts = [
            "setoption",
            "name",
            SET_POSITION_VALUE,
            "value",
            "100",
            "fen",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::SetPosVal(PosValueOpt::Val {
                val: 100,
                fen: "fen".into()
            }))
        );

        let opts = [
            "setoption",
            "name",
            SET_POSITION_VALUE,
            "value",
            "100",
            "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8",
            "b",
            "-",
            "-",
            "40",
            "50",
        ];
        assert_eq!(
            SetOpt::try_from(&opts[..]),
            Ok(SetOpt::SetPosVal(PosValueOpt::Val {
                val: 100,
                fen: FEN_STR.into(),
            }))
        );
    }
}

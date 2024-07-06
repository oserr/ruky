// En enum to represent all errors in the library.
#[derive(Debug, Clone, PartialEq)]
pub enum UziErr {
    BadBool,
    BadMillis(String, String),
    BadNumber(String),
    BadOpponent,
    BadPlayerType,
    BadPositionVal,
    BadTitle,
    GoErr,
    MissingCmd,
    MissingOnOff,
    NothingSetForGo,
    ParseSqErr(String),
    ParsePieceErr(String),
    Position,
    SetOptErr,
    UnknownOpt,
    What,
}

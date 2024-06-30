// En enum to represent all errors in the library.
#[derive(Debug, Clone, PartialEq)]
pub enum UziErr {
    BadBool,
    BadMillis(String, String),
    BadNumber,
    GoErr,
    MissingCmd,
    MissingOnOff,
    NothingSetForGo,
    Position,
    SetOptErr,
    UnknownOpt,
    What,
}

// En enum to represent all errors in the library.
#[derive(Debug, Clone, PartialEq)]
pub enum UziErr {
    MissingCmd,
    MissingOnOff,
    NothingSetForGo,
    Position,
    What,
}

// TODO: move all other errors here to have a single error enum.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RukyErr {
    SearchErr,
    SearchChooseNext,
    SearchMissingBoard,
    SearchTerminalBoard,
    Decoding,
    NoMovesButExpected,
    MoveTensorDim,
    EvalTensorDim,
    InputIsNotValid,
    PreconditionErr,
}

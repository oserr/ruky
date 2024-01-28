#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Piece<T> {
    King(T),
    Queen(T),
    Rook(T),
    Bishop(T),
    Knight(T),
    Pawn(T),
}

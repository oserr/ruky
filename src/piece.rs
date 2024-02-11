#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Piece<T> {
    King(T),
    Queen(T),
    Rook(T),
    Bishop(T),
    Knight(T),
    Pawn(T),
}

impl<T> Piece<T> {
    pub fn is_king(&self) -> bool {
        match *self {
            Piece::King(_) => true,
            _ => false,
        }
    }

    pub fn is_queen(&self) -> bool {
        match *self {
            Piece::Queen(_) => true,
            _ => false,
        }
    }

    pub fn is_rook(&self) -> bool {
        match *self {
            Piece::Rook(_) => true,
            _ => false,
        }
    }

    pub fn is_bishop(&self) -> bool {
        match *self {
            Piece::Bishop(_) => true,
            _ => false,
        }
    }

    pub fn is_knight(&self) -> bool {
        match *self {
            Piece::Knight(_) => true,
            _ => false,
        }
    }

    pub fn is_pawn(&self) -> bool {
        match *self {
            Piece::Pawn(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    White,
    Black,
}

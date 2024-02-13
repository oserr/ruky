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

    // Maps the self into the same Piece but containing a different value.
    pub fn with<U>(&self, val: U) -> Piece<U> {
        match *self {
            Piece::King(_) => Piece::King(val),
            Piece::Queen(_) => Piece::Queen(val),
            Piece::Rook(_) => Piece::Rook(val),
            Piece::Bishop(_) => Piece::Bishop(val),
            Piece::Knight(_) => Piece::Knight(val),
            Piece::Pawn(_) => Piece::Pawn(val),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    White,
    Black,
}

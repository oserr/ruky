use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
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
        matches!(*self, Piece::King(_))
    }

    pub fn is_queen(&self) -> bool {
        matches!(*self, Piece::Queen(_))
    }

    pub fn is_rook(&self) -> bool {
        matches!(*self, Piece::Rook(_))
    }

    pub fn is_bishop(&self) -> bool {
        matches!(*self, Piece::Bishop(_))
    }

    pub fn is_knight(&self) -> bool {
        matches!(*self, Piece::Knight(_))
    }

    pub fn is_pawn(&self) -> bool {
        matches!(*self, Piece::Pawn(_))
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

    // Shorthand for returning the piece with the unit type.
    pub fn kind(&self) -> Piece<()> {
        self.with(())
    }

    // Returns payload in the Piece.
    pub fn val(&self) -> T
    where
        T: Clone,
    {
        let val = match self {
            Piece::King(ref v) => v,
            Piece::Queen(ref v) => v,
            Piece::Rook(ref v) => v,
            Piece::Bishop(ref v) => v,
            Piece::Knight(ref v) => v,
            Piece::Pawn(ref v) => v,
        };
        val.clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn is_white(&self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false,
        }
    }

    pub fn is_black(&self) -> bool {
        !self.is_white()
    }
}

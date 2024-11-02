use crate::board::Board;
use crate::piece::Piece;
use crate::ruky::Ruky;
use std::cell::RefCell;
use uzi::eng::Eng;
use uzi::engtx::UziOut;
use uzi::err::UziErr;
use uzi::guicmd::{Go, Pos, PosOpt};
use uzi::piece::Piece as UziPiece;

#[derive(Clone, Debug)]
struct RandomEng {
    ruky: Ruky,
    uzi_out: UziOut,
    board: RefCell<Option<Board>>,
}

impl Eng for RandomEng {
    fn position(&mut self, pos: &Pos) -> Result<(), UziErr> {
        let mut board = match pos.pos {
            PosOpt::StartPos => self.ruky.new_board(),
            PosOpt::Fen(ref fen) => self.ruky.from_fen(fen).map_err(|_| UziErr::Position)?,
        };
        if pos.moves.is_some() {
            // Convert the Uzi moves to moves that Ruky understands.
            let moves: Vec<(u8, u8, Option<Piece<()>>)> = pos
                .moves
                .as_ref()
                .unwrap()
                .iter()
                .filter(|pm| !pm.is_null())
                .map(|pm| {
                    let from_to = pm.from_to().unwrap();
                    (
                        u8::from(from_to.0),
                        u8::from(from_to.1),
                        pm.promo().map(|p| p.into()),
                    )
                })
                .collect();
            board = board.next_from_rc(&moves).ok_or(UziErr::Position)?;
        }
        self.board.borrow_mut().replace(board);
        Ok(())
    }

    fn new_game(&mut self) -> Result<(), UziErr> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), UziErr> {
        Ok(())
    }

    fn go(&mut self, _go_cmd: &Go) -> Result<(), UziErr> {
        todo!()
    }
}

impl From<UziPiece> for Piece<()> {
    fn from(piece: UziPiece) -> Piece<()> {
        match piece {
            UziPiece::King => Piece::King(()),
            UziPiece::Queen => Piece::Queen(()),
            UziPiece::Rook => Piece::Rook(()),
            UziPiece::Bishop => Piece::Bishop(()),
            UziPiece::Knight => Piece::Knight(()),
            UziPiece::Pawn => Piece::Pawn(()),
        }
    }
}

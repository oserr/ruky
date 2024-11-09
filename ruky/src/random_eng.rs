use crate::board::Board;
use crate::piece::Piece;
use crate::piece_move::PieceMove;
use crate::random_search::RandomSearch;
use crate::ruky::Ruky;
use crate::search::Search;
use crate::sq::Sq;
use log;
use std::cell::RefCell;
use std::sync::Arc;
use uzi::eng::Eng;
use uzi::engtx::EngTx;
use uzi::err::UziErr;
use uzi::guicmd::{Go, Pos, PosOpt};
use uzi::piece::Piece as UziPiece;
use uzi::pm::Pm as UziPm;
use uzi::sq::Sq as UziSq;

#[derive(Clone, Debug)]
pub struct RandomEng<T: EngTx> {
    ruky: Ruky,
    uzi_out: Arc<T>,
    board: RefCell<Option<Board>>,
}

impl<T: EngTx> RandomEng<T> {
    pub fn new(uzi_out: Arc<T>) -> Self {
        Self {
            ruky: Ruky::new(),
            uzi_out,
            board: RefCell::new(None),
        }
    }
}

impl<E: EngTx> Eng for RandomEng<E> {
    fn position(&mut self, pos: &Pos) -> Result<(), UziErr> {
        log::info!("Executing command position: {:?}", pos);
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
        log::info!("The position has been set");
        Ok(())
    }

    fn new_game(&mut self) -> Result<(), UziErr> {
        log::info!("Executing command new_game");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), UziErr> {
        log::info!("Executing command stop");
        Ok(())
    }

    fn go(&mut self, _go_cmd: &Go) -> Result<(), UziErr> {
        log::info!("Executing command go");
        // TODO: Make the errors specific and use the args in the go command.
        let binding = self.board.borrow();
        let board = binding.as_ref().ok_or(UziErr::Position)?;
        let search_result = RandomSearch::new()
            .search_board(board)
            .map_err(|_| UziErr::Position)?;
        let best_move = search_result.best_move().ok_or(UziErr::Position)?;
        log::info!("Calculated best move: {:?}", best_move);
        self.uzi_out.send_best(best_move.into());
        Ok(())
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

impl<T> From<Piece<T>> for UziPiece {
    fn from(piece: Piece<T>) -> UziPiece {
        match piece {
            Piece::King(_) => UziPiece::King,
            Piece::Queen(_) => UziPiece::Queen,
            Piece::Rook(_) => UziPiece::Rook,
            Piece::Bishop(_) => UziPiece::Bishop,
            Piece::Knight(_) => UziPiece::Knight,
            Piece::Pawn(_) => UziPiece::Pawn,
        }
    }
}

impl From<Piece<PieceMove>> for UziPm {
    fn from(piece_move: Piece<PieceMove>) -> UziPm {
        match piece_move.val() {
            PieceMove::Simple { from, to }
            | PieceMove::Capture { from, to, .. }
            | PieceMove::EnPassant { from, to, .. } => UziPm::Normal {
                from: from.into(),
                to: to.into(),
            },
            PieceMove::Castle {
                king_from, king_to, ..
            } => UziPm::Normal {
                from: king_from.into(),
                to: king_to.into(),
            },
            PieceMove::Promo { from, to, promo }
            | PieceMove::PromoCap {
                from, to, promo, ..
            } => UziPm::Promo {
                from: from.into(),
                to: to.into(),
                promo: promo.into(),
            },
        }
    }
}

impl From<Sq> for UziSq {
    fn from(sq: Sq) -> UziSq {
        UziSq::new(sq.into())
    }
}

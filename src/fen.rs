// This module contains code to parse FEN strings, i.e. strings in
// Forsyth-Edwards Notation, which is used to encode the state of a chess
// position in a one-line ascii string. A fen string consists of 6 fields
// separated by a whitespace:
// - the piece placment
// - the side to move
// - castling rights
// - en passant target square
// - the half move clock
// - the full move counter
//
// For example:
// - [rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1] describes the
//   starting position.
// - [rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1] after 1. e4.
// - [rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2] after 1. e4
//   c5.
// - [rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2] after 1.
//   e4 c5 2. Nf3.
//
// For more background, see https://www.chessprogramming.org/Forsyth-Edwards_Notation.
use crate::board::{Board, BoardBuilder};
use crate::piece::Color;
use crate::piece_set::PiecesErr;
use crate::sq::Sq;

const NUM_FIELDS: usize = 6;

// from_fen constructs a Board from a fen string.
//
// @param fen The fen string.
// @param builder A board builder for building the board.
// @return a Result with a Board or a FenErr if there is an error either parsing
// the string or building the board.
pub(crate) fn from_fen(fen: &str, mut builder: BoardBuilder) -> Result<Board, FenErr> {
    let split_iter = fen.trim().split(' ');
    let num_fields = split_iter.clone().count();

    if num_fields < NUM_FIELDS {
        return Err(FenErr::NotEnoughFields);
    } else if num_fields > NUM_FIELDS {
        return Err(FenErr::TooManyFields);
    }

    for (i, field) in split_iter.enumerate() {
        match i {
            0 => parse_pieces(field, &mut builder)?,
            1 => {
                match field {
                    "w" => builder.set_color(Color::White),
                    "b" => builder.set_color(Color::Black),
                    _ => return Err(FenErr::BadColor(field.to_string())),
                };
            }
            2 => parse_castling(field, &mut builder)?,
            3 => parse_passant(field, &mut builder)?,
            4 => {
                let half_move = field
                    .parse::<u16>()
                    .map_err(|_| FenErr::BadHalfMove(field.to_string()))?;
                if half_move > 50 {
                    return Err(FenErr::BadHalfMove(field.to_string()));
                }
                builder.set_half_move(half_move);
            }
            5 => {
                let full_move = field
                    .parse::<u16>()
                    .map_err(|_| FenErr::BadFullMove(field.to_string()))?;
                builder.set_full_move(std::cmp::max(full_move, 1));
            }
            _ => panic!("Should never get here."),
        };
    }

    builder.build().map_err(From::<PiecesErr>::from)
}

// parse_pieces parses the pieces field in a FEN string.
//
// @param field The field containing the pieces.
// @param builder A board builder to set the pieces.
// @return a Result with a unit or a FenErr if there is an error parsing the
// pieces field.
fn parse_pieces(field: &str, builder: &mut BoardBuilder) -> Result<(), FenErr> {
    // A counter for the current square.
    let mut s = 0u32;

    // Reverse the rows so we can start at 0.
    for row in field.split('/').rev() {
        for letter in row.chars() {
            // Numbers indicate empty squares.
            if ('1'..='8').contains(&letter) {
                s += letter.to_digit(10).unwrap();
                continue;
            }

            let sq = Sq::from(s);

            match letter {
                'K' => builder.white_king(sq),
                'Q' => builder.white_queen(sq),
                'R' => builder.white_rook(sq),
                'B' => builder.white_bishop(sq),
                'N' => builder.white_knight(sq),
                'P' => builder.white_pawn(sq),
                'k' => builder.black_king(sq),
                'q' => builder.black_queen(sq),
                'r' => builder.black_rook(sq),
                'b' => builder.black_bishop(sq),
                'n' => builder.black_knight(sq),
                'p' => builder.black_pawn(sq),
                _ => return Err(FenErr::BadPiece(letter)),
            };
            s += 1;
        }
    }

    // We should have 64 total squares when we are done processing the pieces.
    if s != 64 {
        return Err(FenErr::BadSqCount(s));
    }

    Ok(())
}

// parse_castling parses the castling rights field in a FEN string.
//
// @param field The field containing the castling rights.
// @param builder A board builder to set the castling rights.
// @return a Result with a unit or a FenErr if there is an error parsing the
// pieces field.
fn parse_castling(field: &str, builder: &mut BoardBuilder) -> Result<(), FenErr> {
    if field == "-" {
        return Ok(());
    }

    if field.len() > 4 {
        return Err(FenErr::BadCastling(field.to_string()));
    }

    for letter in field.chars() {
        match letter {
            'K' => builder.white_king_castle(true),
            'Q' => builder.white_queen_castle(true),
            'k' => builder.black_king_castle(true),
            'q' => builder.black_queen_castle(true),
            _ => return Err(FenErr::BadCastlingToken(letter)),
        };
    }

    Ok(())
}

// parse_passant parses the en-passant field in a FEN string.
//
// @param field The field containing the en-passant target square.
// @param builder A board builder to set the en-passant.
// @return a Result with a unit or a FenErr if there is an error parsing the
// pieces field.
fn parse_passant(field: &str, builder: &mut BoardBuilder) -> Result<(), FenErr> {
    match field.chars().count() {
        1 => {
            if field != "-" {
                return Err(FenErr::BadPassant(field.to_string()));
            }
            Ok(())
        }
        2 => {
            let mut citer = field.chars();
            let col_letter = citer.next().unwrap();
            let row_letter = citer.next().unwrap();

            if !('a'..='h').contains(&col_letter) || !('1'..='8').contains(&row_letter) {
                return Err(FenErr::BadPassant(field.to_string()));
            }

            let col = col_letter as u8 - b'a';
            let row = row_letter as u8 - b'1';

            let target = Sq::from_rc(row, col).unwrap();

            builder.set_passant(target);
            Ok(())
        }
        _ => Err(FenErr::BadPassant(field.to_string())),
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum FenErr {
    // These represent format errors in the FEN string.
    #[error("not enough fields")]
    NotEnoughFields,
    #[error("too many fields")]
    TooManyFields,
    #[error("invalid square count {0}")]
    BadSqCount(u32),
    #[error("piece {0} is not valid")]
    BadPiece(char),
    #[error("half move count {0} is not valid")]
    BadHalfMove(String),
    #[error("full move count {0} is not valid")]
    BadFullMove(String),
    #[error("color {0} is not valid")]
    BadColor(String),
    #[error("castling {0} is not valid")]
    BadCastling(String),
    #[error("castling token {0} is not valid")]
    BadCastlingToken(char),
    #[error("en-passant square {0} is not valid")]
    BadPassant(String),

    // These represent logical errors in Board position, and map one-to-one to PiecesErr.
    #[error("pieces need a king")]
    NoKing,
    #[error("too many queens")]
    TooManyQueens,
    #[error("too many rooks")]
    TooManyRooks,
    #[error("too many bishops")]
    TooManyBishops,
    #[error("too many knights")]
    TooManyKnights,
    #[error("too many pawns")]
    TooManyPawns,
    #[error("invalid castling rights")]
    BadCastle,
}

// Conversion from PiecesErr to FenErr.
impl From<PiecesErr> for FenErr {
    fn from(err: PiecesErr) -> FenErr {
        match err {
            PiecesErr::NoKing => FenErr::NoKing,
            PiecesErr::TooManyQueens => FenErr::TooManyQueens,
            PiecesErr::TooManyRooks => FenErr::TooManyRooks,
            PiecesErr::TooManyBishops => FenErr::TooManyBishops,
            PiecesErr::TooManyKnights => FenErr::TooManyKnights,
            PiecesErr::TooManyPawns => FenErr::TooManyPawns,
            PiecesErr::BadCastle => FenErr::BadCastle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::BitBoard;
    use crate::magics::ChessMagics;
    use crate::sq;
    use lazy_static::lazy_static;
    use std::sync::Arc;

    lazy_static! {
        static ref MAGICS: Arc<ChessMagics> = Arc::new(
            ChessMagics::from_precomputed().expect("Unable to compute magics for unit test.")
        );
    }

    #[test]
    fn not_enough_fields() {
        assert_eq!(
            from_fen("", BoardBuilder::from(MAGICS.clone())),
            Err(FenErr::NotEnoughFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::NotEnoughFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::NotEnoughFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::NotEnoughFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::NotEnoughFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::NotEnoughFields)
        );
    }

    #[test]
    fn too_many_fields() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 x",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::TooManyFields)
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 x y",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::TooManyFields)
        );
    }

    #[test]
    fn bad_square_count() {
        assert_eq!(
            from_fen(
                "r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadSqCount(57))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/NBQKBNR w KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadSqCount(63))
        );
    }

    #[test]
    fn bad_piece() {
        assert_eq!(
            from_fen(
                "Y7/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadPiece('Y'))
        );
    }

    #[test]
    fn bad_color() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR White KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadColor("White".into()))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR wb KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadColor("wb".into()))
        );
    }

    #[test]
    fn bad_castling() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkqKQ - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadCastling("KQkqKQ".into()))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w zQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadCastlingToken('z'))
        );
    }

    #[test]
    fn bad_passant() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e33 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadPassant("e33".into()))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq x3 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadPassant("x3".into()))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e9 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadPassant("e9".into()))
        );
    }

    #[test]
    fn bad_half_move() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - half 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadHalfMove("half".into()))
        );
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 55 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadHalfMove("55".into()))
        );
    }

    #[test]
    fn bad_full_move() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 full",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadFullMove("full".into()))
        );

        // Technially, a chess game can go on forever with infinite moves, but we assume
        // that no game lasts more than u16::MAX moves.
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 111111111111",
                BoardBuilder::from(MAGICS.clone())
            ),
            Err(FenErr::BadFullMove("111111111111".into()))
        );
    }

    #[test]
    fn init_position() {
        assert_eq!(
            from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                BoardBuilder::from(MAGICS.clone())
            ),
            Ok(Board::from(MAGICS.clone()))
        );
    }

    #[test]
    fn pawns_and_kings_fen() {
        let board = from_fen(
            "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 40 50",
            BoardBuilder::from(MAGICS.clone()),
        )
        .unwrap();

        let zero = BitBoard::new();

        assert!(board.color().is_black());
        assert!(!board.has_wk_castle());
        assert!(!board.has_wq_castle());
        assert!(!board.has_bk_castle());
        assert!(!board.has_bq_castle());
        assert_eq!(board.half_moves(), 40);
        assert_eq!(board.full_moves(), 50);

        assert_eq!(
            board.white_pawns(),
            BitBoard::from(&[sq::A3, sq::F3, sq::B4, sq::E4, sq::H4, sq::D5])
        );
        assert_eq!(board.white_king(), BitBoard::from(sq::H3));
        assert_eq!(board.white_queens(), zero);
        assert_eq!(board.white_rooks(), zero);
        assert_eq!(board.white_bishops(), zero);
        assert_eq!(board.white_knights(), zero);

        assert_eq!(
            board.black_pawns(),
            BitBoard::from(&[sq::A4, sq::F4, sq::B5, sq::E5, sq::H5, sq::D6])
        );
        assert_eq!(board.black_king(), BitBoard::from(&[sq::F7]));
        assert_eq!(board.black_queens(), zero);
        assert_eq!(board.black_rooks(), zero);
        assert_eq!(board.black_bishops(), zero);
        assert_eq!(board.black_knights(), zero);
    }
}

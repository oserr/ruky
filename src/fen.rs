use crate::board::{Board, BoardBuilder};
use crate::piece::Color;
use crate::sq::Sq;

pub enum FenErr {
    NotEnoughFields,
    NotEnoughSquares,
    TooManyFields,
    BadPiece(char),
    BadHalfMove(String),
    BadFullMove(String),
    BadSetup,
    BadColor(String),
    BadCastling(String),
    BadCastlingToken(char),
    BadPassant(String),
}

const NUM_FIELDS: usize = 6;

pub(crate) fn from_fen(fen: &str, mut builder: BoardBuilder) -> Result<Board, FenErr> {
    let split_iter = fen.split(' ');
    let (num_fields, _) = split_iter.size_hint();

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
                builder.set_half_move(half_move);
            }
            5 => {
                let full_move = field
                    .parse::<u16>()
                    .map_err(|_| FenErr::BadFullMove(field.to_string()))?;
                builder.set_full_move(full_move);
            }
            _ => panic!("Should never get here."),
        };
    }

    builder.build().map_err(|_| FenErr::BadSetup)
}

fn parse_pieces(field: &str, builder: &mut BoardBuilder) -> Result<(), FenErr> {
    let mut s = 0;
    for row in field.split('/').rev() {
        for letter in row.chars() {
            if ('1'..'8').contains(&letter) {
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

    if s != 64 {
        return Err(FenErr::NotEnoughSquares);
    }

    Ok(())
}

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

            let col = col_letter as u8 - 'a' as u8;
            let row = row_letter as u8 - '1' as u8;

            let target = Sq::from_rc(row, col).unwrap();

            builder.set_passant(target);
            Ok(())
        }
        _ => Err(FenErr::BadPassant(field.to_string())),
    }
}

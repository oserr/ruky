use crate::bitboard::{BitBoard, RANK_3, RANK_6};
use crate::magics::ChessMagics;
use crate::piece::{Color, Piece, Piece::*};
use crate::piece_move::{PieceMove, PieceMove::*};
use crate::piece_set::{AttackSquares, PieceSet, PiecesErr, PsBuilder};
use crate::sq::Sq;
use std::sync::Arc;

// Represents a chess board, and encodes the rules for moving pieces and
// determining the current game state, e.g. whether the game is drawn.
#[derive(Clone)]
pub struct Board {
    // The board state. We use a Box for it because this makes it much cheaper to move a board.
    state: Box<BoardState>,

    // We use an Arc for ChessMagics, because ChessMagics are expensive to compute, and hence we
    // want to share one instance of chess magics where ever they are needed, and between threads.
    magics: Arc<ChessMagics>,
}

impl Board {
    ///////////////////////////
    // Getters for board state.
    ///////////////////////////

    #[inline]
    pub fn white_king(&self) -> BitBoard {
        self.white().king()
    }

    #[inline]
    pub fn white_queens(&self) -> BitBoard {
        self.white().queens()
    }

    #[inline]
    pub fn white_rooks(&self) -> BitBoard {
        self.white().rooks()
    }

    #[inline]
    pub fn white_bishops(&self) -> BitBoard {
        self.white().bishops()
    }

    #[inline]
    pub fn white_knights(&self) -> BitBoard {
        self.white().knights()
    }

    #[inline]
    pub fn white_pawns(&self) -> BitBoard {
        self.white().pawns()
    }

    #[inline]
    pub fn black_king(&self) -> BitBoard {
        self.black().king()
    }

    #[inline]
    pub fn black_queens(&self) -> BitBoard {
        self.black().queens()
    }

    #[inline]
    pub fn black_rooks(&self) -> BitBoard {
        self.black().rooks()
    }

    #[inline]
    pub fn black_bishops(&self) -> BitBoard {
        self.black().bishops()
    }

    #[inline]
    pub fn black_knights(&self) -> BitBoard {
        self.black().knights()
    }

    #[inline]
    pub fn black_pawns(&self) -> BitBoard {
        self.black().pawns()
    }

    #[inline]
    pub fn has_wk_castle(&self) -> bool {
        self.white().has_king_castle()
    }

    #[inline]
    pub fn has_wq_castle(&self) -> bool {
        self.white().has_queen_castle()
    }

    #[inline]
    pub fn has_bk_castle(&self) -> bool {
        self.black().has_king_castle()
    }

    #[inline]
    pub fn has_bq_castle(&self) -> bool {
        self.black().has_queen_castle()
    }

    #[inline]
    fn white(&self) -> &PieceSet {
        match self.color() {
            Color::White => &self.state.mine,
            Color::Black => &self.state.other,
        }
    }

    #[inline]
    fn black(&self) -> &PieceSet {
        match self.color() {
            Color::Black => &self.state.mine,
            Color::White => &self.state.other,
        }
    }

    #[inline]
    pub fn color(&self) -> Color {
        self.state.color()
    }

    #[inline]
    pub fn half_moves(&self) -> u16 {
        self.state.half_move
    }

    #[inline]
    pub fn full_moves(&self) -> u16 {
        self.state.full_move
    }

    #[inline]
    fn passant(&self) -> Option<PassantSq> {
        self.state.passant_sq
    }

    // Updates the game state.
    fn update_game_state(&mut self, piece_move: Option<PieceMove>) {
        if piece_move.is_some() && piece_move.unwrap().is_king_capture() {
            self.state.game_state = GameState::Mate(self.state.color());
            return;
        }

        if self.state.half_move >= 50 || !self.state.is_enough_material() {
            self.state.game_state = GameState::Draw;
            return;
        }

        let moves = self.all_moves();
        if moves.is_none() {
            self.state.game_state = GameState::Draw;
            return;
        }

        // We want to verify that we have some move such that we are not in check.
        for pmv in moves.unwrap() {
            let mut board = self.clone();
            board.state.partial_update(pmv, self.magics.as_ref());
            if !board.state.is_other_in_check() {
                self.state.game_state = if self.is_check() {
                    GameState::Check(self.state.color())
                } else {
                    GameState::Next(self.state.color())
                };
                return;
            }
        }

        self.state.game_state = if self.is_check() {
            GameState::Mate(self.state.color())
        } else {
            // If we don't have moves without check, but are not currently in check, then we
            // are in stalemate.
            GameState::Draw
        };
    }

    // Updates the board state after a move is made.
    fn update(&mut self, piece_move: Piece<PieceMove>) {
        self.state.partial_update(piece_move, self.magics.as_ref());
        self.update_game_state(Some(piece_move.val()));
        self.state.prev_moves.push(piece_move);
    }

    // Returns boards representing all the valid positions that are reachable from
    // the current position. If this is a terminal state, then it returns None.
    pub fn next_boards(&self) -> Option<Vec<Board>> {
        if self.is_terminal() {
            return None;
        }

        Some(
            self.all_moves()
                .expect("Unable to compute any moves in a non-terminal state.")
                .into_iter()
                .map(|piece_move| {
                    let mut board = self.clone();
                    board.update(piece_move);
                    board
                })
                .filter(|board| !board.state.is_other_in_check())
                .collect(),
        )
    }

    // Returns the legal moves from the current position. If the board is in a
    // terminal state, then it returns None.
    pub fn next_moves(&self) -> Option<Vec<Piece<PieceMove>>> {
        if self.is_terminal() {
            return None;
        }

        Some(
            self.all_moves()
                .expect("Unable to compute any moves in a non-terminal state.")
                .into_iter()
                .filter(|piece_move| {
                    let mut board = self.clone();
                    board.update(*piece_move);
                    !board.state.is_other_in_check()
                })
                .collect(),
        )
    }

    // Returns true if the current position represents a terminal state.
    #[inline]
    pub fn is_terminal(&self) -> bool {
        self.state.game_state.is_terminal()
    }

    // Returns true if the player moving next is in check.
    #[inline]
    pub fn is_check(&self) -> bool {
        self.state.is_mine_in_check()
    }

    // Getter for the game state.
    #[inline]
    pub fn game_state(&self) -> GameState {
        self.state.game_state
    }

    // Computes all the moves, including moves that are not legal, e.g. putting
    // oneself in check. If there are no moves to be made, e.g. we're already in
    // a terminal state, then it return None.
    pub fn all_moves(&self) -> Option<Vec<Piece<PieceMove>>> {
        if self.state.game_state.is_terminal() {
            return None;
        }

        let mut moves: Vec<Piece<PieceMove>> = Vec::new();

        self.king_moves(&mut moves);
        self.queen_moves(&mut moves);
        self.rook_moves(&mut moves);
        self.bishop_moves(&mut moves);
        self.knight_moves(&mut moves);
        self.pawn_moves(&mut moves);

        if moves.is_empty() {
            None
        } else {
            Some(moves)
        }
    }

    fn king_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(King(self.state.mine.king()), moves, |b| b.king_moves());
        let (king_castle, queen_castle) = self
            .state
            .mine
            .castle(&self.state.other, self.state.other_attacks.all());
        if let Some(mv) = king_castle {
            moves.push(mv);
        }
        if let Some(mv) = queen_castle {
            moves.push(mv);
        }
    }

    fn queen_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Queen(self.state.mine.queens()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .qmagics(from, self.state.all())
                .expect("Unable to to compute queen magics")
        });
    }

    fn rook_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Rook(self.state.mine.rooks()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .rmagics(from, self.state.all())
                .expect("Unable to to compute rook magics")
        });
    }

    fn bishop_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Bishop(self.state.mine.bishops()), moves, |b| {
            let from = b.first_bit().expect("BitBoard should have a bit set.");
            self.magics
                .bmagics(from, self.state.all())
                .expect("Unable to to compute bishop magics")
        });
    }

    fn knight_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        self.simple_moves(Knight(self.state.mine.knights()), moves, |b| {
            b.knight_moves()
        });
    }

    fn pawn_moves(&self, moves: &mut Vec<Piece<PieceMove>>) {
        if self.state.color().is_white() {
            self.add_pawn_moves(
                moves,
                |bits, empty| bits.wp_moves(empty),
                |s| s.in_last_rank(),
            );
        } else {
            self.add_pawn_moves(
                moves,
                |bits, empty| bits.bp_moves(empty),
                |s| s.in_first_rank(),
            );
        }
    }

    fn add_pawn_moves(
        &self,
        moves: &mut Vec<Piece<PieceMove>>,
        moves_fn: impl Fn(BitBoard, BitBoard) -> (BitBoard, BitBoard),
        is_promo: impl Fn(Sq) -> bool,
    ) {
        let pawns = self.state.mine.pawns();
        let empty = self.state.none();
        let other = self.state.other.all();

        for (from, pawn_bit) in pawns.sq_bit_iter() {
            let (forward_moves, all_attacks) = moves_fn(pawn_bit, empty);

            for to in forward_moves.sq_iter() {
                if is_promo(to) {
                    add_promo(from, to, moves);
                } else {
                    moves.push(Pawn(Simple { from, to }));
                }
            }

            let attacks = all_attacks & other;

            for to in attacks.sq_iter() {
                let cap = self
                    .state
                    .other
                    .find_type(to)
                    .expect("Unable to find a piece for capture.");

                if is_promo(to) {
                    add_promo_with_cap(from, to, cap, moves);
                } else {
                    moves.push(Pawn(Capture { from, to, cap }));
                }
            }

            if let Some(passant_cap) = self
                .state
                .passant_sq
                .and_then(|ps| ps.by_enpassant(from, all_attacks))
            {
                moves.push(passant_cap)
            }
        }
    }

    fn simple_moves(
        &self,
        piece: Piece<BitBoard>,
        moves: &mut Vec<Piece<PieceMove>>,
        move_fn: impl Fn(BitBoard) -> BitBoard,
    ) {
        for (from, bit) in piece.val().sq_bit_iter() {
            let bit_moves = move_fn(bit);

            let non_attacks = bit_moves & self.state.none();
            for to in non_attacks.sq_iter() {
                moves.push(piece.with(Simple { from, to }));
            }

            let attacks = bit_moves & self.state.other.all();
            for to in attacks.sq_iter() {
                let cap = self
                    .state
                    .other
                    .find_type(to)
                    .expect("Unable to find an attack piece.");
                moves.push(piece.with(Capture { from, to, cap }));
            }
        }
    }
}

// Converts ChessMagics into a Board.
impl From<Arc<ChessMagics>> for Board {
    fn from(magics: Arc<ChessMagics>) -> Board {
        let state = Box::<BoardState>::default();
        Board { state, magics }
    }
}

// When comparing boards, we don't care about the magics.
impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl Eq for Board {}

// Only hash the BoardState, not the magics.
impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
}

// For debug, don't include magics.
impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.state)
    }
}

// BoardState holds all the state needed needed to a play a game of regular
// chess, including the position of the pieces, position of squares that are
// attacked, the current game state, the number of half moves, the number of
// full moves, and whether there is an opportunity to capture by en passant.
// Note that castling rights are encoded in the PieceSets.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BoardState {
    // The pieces that are moving next. We use a Box for the pieces other because this
    // makes it much cheaper to swap the pieces after a move is made. This simplifies a lot of code
    // because we can do things in terms of the player moving next.
    mine: Box<PieceSet>,

    // The pieces that are not moving next.
    other: Box<PieceSet>,

    // All positions on the board that are attacked by the pieces moving next.
    my_attacks: AttackSquares,

    // All positions on the board that are attacked by the pieces not moving.
    other_attacks: AttackSquares,

    // The current game state.
    game_state: GameState,

    // The number of half moves, which is used to determine if a game should be drawn because of
    // insufficient progress, i.e. no pawn moves or captures.
    half_move: u16,

    // The number of full moves. In theory, you can have an infinite number of moves in a game, but
    // in practice the game is drawn at some point of there is no progress.
    full_move: u16,

    // If set, represents the square where capture by en-passant is possible.
    passant_sq: Option<PassantSq>,

    // The previous moves leading up to the current board state.
    prev_moves: Vec<Piece<PieceMove>>,
}

impl BoardState {
    // Returns the union of all pieces as a BitBoard.
    fn all(&self) -> BitBoard {
        self.mine.all() | self.other.all()
    }

    // Returns the set of all empty squares.
    fn none(&self) -> BitBoard {
        !self.all()
    }

    // Returns the color moving next.
    fn color(&self) -> Color {
        self.mine.color()
    }

    // Returns the true if pieces moving next are in check.
    fn is_mine_in_check(&self) -> bool {
        (self.mine.king() & self.other_attacks.pieces).any()
    }

    // Returns true if the pieces not moving next are are in check. Technically,
    // this not a a valid state, because a player cannot put themselves in
    // check, which is exactly the motivation for defining this.
    fn is_other_in_check(&self) -> bool {
        (self.other.king() & self.my_attacks.pieces).any()
    }

    // Returns true if there is enough material on either side for a win, or false
    // if neither side can win given the material. The
    // following scenarios are a draw:
    // - only king per side
    // - bishop or knight vs bishop or knight
    fn is_enough_material(&self) -> bool {
        let my_count = self.mine.all().count();
        let other_count = self.other.all().count();

        if my_count > 2 || other_count > 2 {
            return true;
        }

        if my_count == 1 && other_count == 1 {
            return false;
        } else if my_count == 2 && other_count == 1 {
            return self.mine.bishops().count() == 0 && self.mine.knights().count() == 0;
        } else if my_count == 1 && other_count == 2 {
            return self.other.bishops().count() == 0 && self.other.knights().count() == 0;
        }
        // TODO: king + bishop vs king + bishop is draw if bishops are same color.

        true
    }

    fn update_attacks(&mut self, magics: &ChessMagics) {
        self.my_attacks = self.mine.attacks(&self.other, magics);
        self.other_attacks = self.other.attacks(&self.mine, magics);
    }

    // Handles all of the state update after a move is made, except setting the
    // GameState. Some of the state change is subsequently used to compute the final
    // game state.
    fn partial_update(&mut self, piece_move: Piece<PieceMove>, magics: &ChessMagics) {
        let mv = piece_move.val();
        let is_pawn = piece_move.is_pawn();
        let is_cap = mv.is_capture();

        self.mine
            .apply_move(piece_move)
            .expect("Unable to update mine with move.");

        if is_pawn || is_cap {
            self.half_move = 0;
            if is_cap {
                self.other
                    .remove_captured(mv)
                    .expect("Unable to remove captured piece.");
            }
        } else {
            self.half_move += 1;
        }

        if self.color().is_black() {
            self.full_move += 1;
        }

        self.passant_sq = if is_pawn {
            match mv {
                Simple { from, to } => PassantSq::try_passant(from, to, self.color()),
                _ => None,
            }
        } else {
            None
        };

        std::mem::swap(&mut self.mine, &mut self.other);
        self.update_attacks(magics);
    }
}

// Initializes the BoardState for a new game.
impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            mine: Box::new(PieceSet::init_white()),
            other: Box::new(PieceSet::init_black()),
            my_attacks: AttackSquares {
                pieces: BitBoard::new(),
                no_pieces: RANK_3,
            },
            other_attacks: AttackSquares {
                pieces: BitBoard::new(),
                no_pieces: RANK_6,
            },
            game_state: GameState::Next(Color::White),
            half_move: 0,
            full_move: 1,
            passant_sq: None,
            prev_moves: Vec::new(),
        }
    }
}

// A helper class to build chess boards.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BoardBuilder {
    // A piece set builder for white pieces.
    white_builder: PsBuilder,
    // A piece set builder for black pieces.
    black_builder: PsBuilder,
    magics: Arc<ChessMagics>,
    color: Color,
    half_move: u16,
    full_move: u16,
    passant_sq: Option<PassantSq>,
}

impl BoardBuilder {
    ////////////////////////////
    // Setters for white pieces.
    ////////////////////////////

    pub fn white_king(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.set_king(sq);
        self
    }

    pub fn white_queen(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.add_queen(sq);
        self
    }

    pub fn white_rook(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.add_rook(sq);
        self
    }

    pub fn white_bishop(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.add_bishop(sq);
        self
    }

    pub fn white_knight(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.add_knight(sq);
        self
    }

    pub fn white_pawn(&mut self, sq: Sq) -> &mut Self {
        self.white_builder.add_pawn(sq);
        self
    }

    pub fn white_king_castle(&mut self, can_castle: bool) -> &mut Self {
        self.white_builder.set_king_castle(can_castle);
        self
    }

    pub fn white_queen_castle(&mut self, can_castle: bool) -> &mut Self {
        self.white_builder.set_queen_castle(can_castle);
        self
    }

    ////////////////////////////
    // Setters for black pieces.
    ////////////////////////////

    pub fn black_king(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.set_king(sq);
        self
    }

    pub fn black_queen(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.add_queen(sq);
        self
    }

    pub fn black_rook(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.add_rook(sq);
        self
    }

    pub fn black_bishop(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.add_bishop(sq);
        self
    }

    pub fn black_knight(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.add_knight(sq);
        self
    }

    pub fn black_pawn(&mut self, sq: Sq) -> &mut Self {
        self.black_builder.add_pawn(sq);
        self
    }

    pub fn black_king_castle(&mut self, can_castle: bool) -> &mut Self {
        self.black_builder.set_king_castle(can_castle);
        self
    }

    pub fn black_queen_castle(&mut self, can_castle: bool) -> &mut Self {
        self.black_builder.set_queen_castle(can_castle);
        self
    }

    ////////////////////////////
    // Setters for shared state.
    ////////////////////////////

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn set_half_move(&mut self, half_move: u16) -> &mut Self {
        self.half_move = half_move;
        self
    }

    pub fn set_full_move(&mut self, full_move: u16) -> &mut Self {
        self.full_move = full_move;
        self
    }

    pub fn set_passant(&mut self, target: Sq) -> &mut Self {
        self.passant_sq = PassantSq::from_target(target);
        self
    }

    pub fn build(&mut self) -> Result<Board, PiecesErr> {
        let (mine, other) = if self.color.is_white() {
            (
                Box::new(self.white_builder.build()?),
                Box::new(self.black_builder.build()?),
            )
        } else {
            (
                Box::new(self.black_builder.build()?),
                Box::new(self.white_builder.build()?),
            )
        };

        let my_attacks = mine.attacks(&other, self.magics.as_ref());
        let other_attacks = other.attacks(&mine, self.magics.as_ref());

        let mut board = Board {
            state: Box::new(BoardState {
                mine,
                other,
                my_attacks,
                other_attacks,
                game_state: GameState::Next(self.color),
                half_move: self.half_move,
                full_move: self.full_move,
                passant_sq: self.passant_sq,
                prev_moves: Vec::new(),
            }),
            magics: self.magics.clone(),
        };

        board.update_game_state(None);

        Ok(board)
    }
}

// Converts ChessMagics into a BoardBuilder.
impl From<Arc<ChessMagics>> for BoardBuilder {
    fn from(magics: Arc<ChessMagics>) -> Self {
        Self {
            white_builder: *PsBuilder::new().set_color(Color::White),
            black_builder: *PsBuilder::new().set_color(Color::Black),
            magics,
            color: Color::White,
            half_move: 0,
            full_move: 1,
            passant_sq: None,
        }
    }
}

// Represents the current game state. Mate and Draw are final game state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameState {
    Next(Color),
    Check(Color),
    Mate(Color),
    Draw,
}

impl GameState {
    // Returns true if this GameState reprsesents a terminal game state, i.e. a draw
    // or check mate.
    pub fn is_terminal(&self) -> bool {
        !matches!(*self, GameState::Next(_) | GameState::Check(_))
    }
}

// A struct to represent the position where en-passant capture is possible.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PassantSq {
    // Represents the actual location of the piece. This is a square in either the 4th or 5th rank.
    pub actual: Sq,

    // Represents the square where the pawn is captured. This is a square in either the 3rd or 6th
    // rank.
    pub capture: Sq,
}

impl PassantSq {
    // Creates a capture by en passant if the set of pawn attacks contains the
    // square where the pawn is captured by en passant, otherwise returns None.
    fn by_enpassant(&self, from: Sq, attacks: BitBoard) -> Option<Piece<PieceMove>> {
        if attacks.has_bit(self.capture) {
            Some(Pawn(EnPassant {
                from,
                to: self.capture,
                passant: self.actual,
            }))
        } else {
            None
        }
    }

    // Takes a pair in the form of (from, to) representing the source and
    // destination squares for a pawn move, and the color of the player moving.
    // If move represents a 2-square forward move, then it returns a PassantSq
    // to represent the fact that capture by en-passant is possible.
    // This may not actually be the case if there are no pawns in right square, but
    // should not be an issue.
    // TODO: need to check the locations of the other pawns to see en passant is
    // possible.
    fn try_passant(from: Sq, to: Sq, color: Color) -> Option<PassantSq> {
        let (from_row, from_col) = from.rc();
        let (to_row, _) = to.rc();
        match (color, from_row, to_row) {
            (Color::White, 1, 3) => Some(PassantSq {
                actual: to,
                capture: Sq::from_rc(2, from_col).unwrap(),
            }),
            (Color::Black, 6, 4) => Some(PassantSq {
                actual: to,
                capture: Sq::from_rc(5, from_col).unwrap(),
            }),
            _ => None,
        }
    }

    // Builds a PassantSq from the target square if the target square is a valid
    // square, otherwise returns None.
    fn from_target(target: Sq) -> Option<PassantSq> {
        let (row, col) = target.rc();
        let actual = match row {
            2 => Sq::from_rc(3, col),
            5 => Sq::from_rc(4, col),
            _ => None,
        };
        actual.map(|a| PassantSq {
            actual: a,
            capture: target,
        })
    }
}

// Utility to add all the different types of promotions.
fn add_promo(from: Sq, to: Sq, moves: &mut Vec<Piece<PieceMove>>) {
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Queen(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Rook(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Bishop(()),
    }));
    moves.push(Pawn(Promo {
        from,
        to,
        promo: Knight(()),
    }));
}

// Utility to add all the different types of promotions.
fn add_promo_with_cap(from: Sq, to: Sq, cap: Piece<()>, moves: &mut Vec<Piece<PieceMove>>) {
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Queen(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Rook(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Bishop(()),
        cap,
    }));
    moves.push(Pawn(PromoCap {
        from,
        to,
        promo: Knight(()),
        cap,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sq;
    use lazy_static::lazy_static;
    use std::collections::HashSet;

    lazy_static! {
        static ref MAGICS: Arc<ChessMagics> = Arc::new(
            ChessMagics::from_precomputed().expect("Unable to compute magics for unit test.")
        );
    }

    #[test]
    fn board_init_from_magics() {
        let board = Board::from(MAGICS.clone());

        assert!(board.state.color().is_white());
        assert_eq!(*board.state.mine, PieceSet::init_white());
        assert_eq!(*board.state.other, PieceSet::init_black());
        assert_eq!(board.state.game_state, GameState::Next(Color::White));
        assert_eq!(board.state.half_move, 0);
        assert_eq!(board.state.full_move, 1);
        assert_eq!(board.state.passant_sq, None);
        assert!(board.state.prev_moves.is_empty());
    }

    #[test]
    fn moves_from_init() {
        let board = Board::from(MAGICS.clone());
        let mut moves: Vec<Piece<PieceMove>> = vec![];

        board.king_moves(&mut moves);
        assert!(moves.is_empty());

        board.queen_moves(&mut moves);
        assert!(moves.is_empty());

        board.rook_moves(&mut moves);
        assert!(moves.is_empty());

        board.bishop_moves(&mut moves);
        assert!(moves.is_empty());

        board.knight_moves(&mut moves);
        assert_eq!(
            HashSet::from_iter(moves.clone()),
            HashSet::from([
                Knight(Simple {
                    from: sq::B1,
                    to: sq::A3
                }),
                Knight(Simple {
                    from: sq::B1,
                    to: sq::C3
                }),
                Knight(Simple {
                    from: sq::G1,
                    to: sq::F3
                }),
                Knight(Simple {
                    from: sq::G1,
                    to: sq::H3
                }),
            ])
        );

        moves.clear();
        board.pawn_moves(&mut moves);
        assert_eq!(
            HashSet::from_iter(moves),
            HashSet::from([
                Pawn(Simple {
                    from: sq::A2,
                    to: sq::A3
                }),
                Pawn(Simple {
                    from: sq::A2,
                    to: sq::A4
                }),
                Pawn(Simple {
                    from: sq::B2,
                    to: sq::B3
                }),
                Pawn(Simple {
                    from: sq::B2,
                    to: sq::B4
                }),
                Pawn(Simple {
                    from: sq::C2,
                    to: sq::C3
                }),
                Pawn(Simple {
                    from: sq::C2,
                    to: sq::C4
                }),
                Pawn(Simple {
                    from: sq::D2,
                    to: sq::D3
                }),
                Pawn(Simple {
                    from: sq::D2,
                    to: sq::D4
                }),
                Pawn(Simple {
                    from: sq::E2,
                    to: sq::E3
                }),
                Pawn(Simple {
                    from: sq::E2,
                    to: sq::E4
                }),
                Pawn(Simple {
                    from: sq::F2,
                    to: sq::F3
                }),
                Pawn(Simple {
                    from: sq::F2,
                    to: sq::F4
                }),
                Pawn(Simple {
                    from: sq::G2,
                    to: sq::G3
                }),
                Pawn(Simple {
                    from: sq::G2,
                    to: sq::G4
                }),
                Pawn(Simple {
                    from: sq::H2,
                    to: sq::H3
                }),
                Pawn(Simple {
                    from: sq::H2,
                    to: sq::H4
                }),
            ])
        );
    }

    #[test]
    fn checkmate1() {
        let mut builder = BoardBuilder::from(MAGICS.clone());

        let board = builder
            .set_color(Color::Black)
            .black_king(sq::A8)
            .black_pawn(sq::A7)
            .white_king(sq::C8)
            .white_knight(sq::C7)
            .build();

        assert!(board.is_ok());

        let board = board.unwrap();
        assert!(board.is_terminal());
        assert_eq!(board.game_state(), GameState::Mate(Color::Black));
        assert_eq!(board.next_boards(), None);
    }

    #[test]
    fn checkmate2() {
        let mut builder = BoardBuilder::from(MAGICS.clone());

        let board = builder
            .set_color(Color::Black)
            .black_king(sq::A8)
            .black_bishop(sq::A7)
            .white_king(sq::C7)
            .white_bishop(sq::H1)
            .build();

        assert!(board.is_ok());

        let board = board.unwrap();
        assert!(board.is_terminal());
        assert_eq!(board.game_state(), GameState::Mate(Color::Black));
    }

    #[test]
    fn check1() {
        let mut builder = BoardBuilder::from(MAGICS.clone());

        let board = builder
            .set_color(Color::Black)
            .black_king(sq::E4)
            .white_king(sq::D2)
            .white_pawn(sq::D3)
            .build();

        assert!(board.is_ok());

        let board = board.unwrap();
        assert_eq!(board.game_state(), GameState::Check(Color::Black));
    }

    #[test]
    fn not_enough_material() {
        // Bishop + king vs king.
        let mut builder = BoardBuilder::from(MAGICS.clone());

        let board = builder
            .set_color(Color::White)
            .black_king(sq::A8)
            .black_bishop(sq::A7)
            .white_king(sq::C8)
            .build();

        assert!(board.is_ok());

        let board = board.unwrap();
        assert_eq!(board.game_state(), GameState::Draw);

        // Knight + king vs king.
        let mut builder = BoardBuilder::from(MAGICS.clone());

        let board = builder
            .set_color(Color::White)
            .black_king(sq::A8)
            .black_knight(sq::A7)
            .white_king(sq::C8)
            .build();

        assert!(board.is_ok());

        let board = board.unwrap();
        assert_eq!(board.game_state(), GameState::Draw);
    }
}

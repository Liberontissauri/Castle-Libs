
use std::{io::{Error, ErrorKind}, u16, time::SystemTime};

use pleco::{Board, BitMove};

pub struct ChessGame {
    inital_board: Board,
    moves: Vec<Move>,
    start_time: u32, // in milliseconds
    time_limit: u32, // in milliseconds
    increment: u32, // in milliseconds
}

impl ChessGame {
    pub fn compute_current_board(&self) -> Board {
        let mut board = self.inital_board.clone();
        for mov in self.moves.iter() {
            board.apply_uci_move(&mov.uci_move); //Assumes all the previous moves were valid
        }
        board
    }
    pub fn compute_board_at_turn(&self, target_turn: u16) -> Board {
        let mut board = self.inital_board.clone();
        for (turn, mov) in self.moves.iter().enumerate() {
            if (turn as u16) < target_turn {
                board.apply_uci_move(&mov.uci_move); //Assumes all the previous moves were valid
            } else {
                break;
            }
        }
        board
    }
    pub fn is_move_legal(&self, mov: &Move) -> bool {
        let mut board = self.compute_current_board();
        let is_legal = board.apply_uci_move(&mov.uci_move);
        return is_legal;
    }
    pub fn play_move(mut self, mov: Move) -> Result<ChessGame, Error>{
        if self.is_move_legal(&mov) {
            self.moves.push(mov);
            Ok(self)
        } else {
            Err(Error::new(ErrorKind::Other, "Tried playing an illegal move"))
        }
    }
    pub fn undo_move(mut self) -> Result<ChessGame, Error> {
        if let Some(mov) = self.moves.pop() {
            let mut board = self.compute_current_board();
            board.undo_move();
            Ok(self)
        } else {
            Err(Error::new(ErrorKind::Other, "Tried undoing a move when there are no moves to undo"))
        }
    }
    pub fn compute_white_moves_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() { 
            turn += 1;
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::Black { // Needs Testing but its because, presumably, turn() tracks the next move, not the current move
                elapsed_time += mov.time_taken;
                if elapsed_time >= self.increment {
                    elapsed_time -= self.increment;
                } else {
                    elapsed_time = 0;
                }
            }
        }
        elapsed_time
    }
    pub fn compute_black_moves_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() {
            turn += 1; // Because the turns must start at 1 for turn 0 to be the initial board state
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::White { // Needs Testing but its because, presumably, turn() tracks the next move, not the current move
                elapsed_time += mov.time_taken;
                if elapsed_time >= self.increment {
                    elapsed_time -= self.increment;
                } else {
                    elapsed_time = 0;
                }
            }
        }
        elapsed_time
    }
    pub fn compute_total_moves_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for mov in self.moves.iter() {
            elapsed_time += mov.time_taken;
        }
        elapsed_time
    }

    pub fn compute_total_elapsed_time(&self) -> u32 {
        let now = SystemTime::now();
        let current_time = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_millis() as u32;
        current_time - self.start_time
    }
    pub fn compute_white_elapsed_time(&self) -> u32 {
        let total_elapsed_time = self.compute_total_elapsed_time();
        let white_moves_time = self.compute_white_moves_time();
        let black_moves_time = self.compute_black_moves_time();
        let turn = self.compute_current_board().turn();
        match turn {
            pleco::Player::White => total_elapsed_time - black_moves_time,
            pleco::Player::Black => white_moves_time,
        }
    }
    pub fn compute_black_elapsed_time(&self) -> u32 {
        let total_elapsed_time = self.compute_total_elapsed_time();
        let white_moves_time = self.compute_white_moves_time();
        let black_moves_time = self.compute_black_moves_time();
        let turn = self.compute_current_board().turn();
        match turn {
            pleco::Player::White => black_moves_time,
            pleco::Player::Black => total_elapsed_time - white_moves_time,
        }
    }

    pub fn is_white_time_over(&self) -> bool {
        let elapsed_time = self.compute_white_elapsed_time();
        elapsed_time > self.time_limit
    }
    pub fn is_black_time_over(&self) -> bool {
        let elapsed_time = self.compute_black_elapsed_time();
        elapsed_time > self.time_limit
    }
    pub fn is_checkmate(&self) -> bool {
        let board = self.compute_current_board();
        board.checkmate()
    }
}

pub struct ChessGameBuilder {
    inital_board: Board,
    moves: Vec<Move>,
    time_limit: u32, // in milliseconds
    increment: u32, // in milliseconds
}
// Get Time since epoch in miliseconds
// let now = SystemTime::now();
// let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
// println!("Time since the epoch: {:?}", since_the_epoch);
impl ChessGameBuilder {
    pub fn new() -> ChessGameBuilder {
        ChessGameBuilder {
            inital_board: Board::start_pos(),
            moves: Vec::new(),
            time_limit: 0,
            increment: 0,
        }
    }
    pub fn with_initial_board(mut self, board: Board) -> ChessGameBuilder {
        self.inital_board = board;
        self
    }
    pub fn with_time_limit(mut self, time_limit: u32) -> ChessGameBuilder {
        self.time_limit = time_limit;
        self
    }
    pub fn with_increment(mut self, increment: u32) -> ChessGameBuilder {
        self.increment = increment;
        self
    }
    pub fn build(self) -> ChessGame {
        let now = SystemTime::now();
        ChessGame {
            inital_board: self.inital_board,
            moves: self.moves,
            start_time: now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_millis() as u32,
            time_limit: self.time_limit,
            increment: self.increment,
        }
    }
    
}

pub struct Move {
    uci_move: String,
    time_taken: u32, // in milliseconds
}
impl Move {
    pub fn new(uci_move: String, time_taken: u32) -> Move {
        Move {
            uci_move,
            time_taken,
        }
    }
}


#[cfg(test)]
mod tests {
    use pleco::Board;
    use crate::chess::*;

    #[test]
    fn single_move_time_elapsed() {
        let mut game = ChessGameBuilder::new()
            .with_time_limit(1000 * 60 * 3)
            .with_increment(10)
            .build();
        let my_move = Move::new(String::from("e2e4"), 1000);
        game = game.play_move(my_move).unwrap();
        assert_eq!(game.compute_white_elapsed_time(), 990);
        assert_eq!(game.compute_black_elapsed_time(), 0);
    }
    #[test]
    fn multiple_move_time_elapsed() {
        let mut game = ChessGameBuilder::new()
            .with_time_limit(1000 * 60 * 3)
            .with_increment(10)
            .build();
        let my_move = Move::new(String::from("e2e4"), 1000);
        game = game.play_move(my_move).unwrap();
        let my_move = Move::new(String::from("a7a6"), 1000);
        game = game.play_move(my_move).unwrap();
        let my_move = Move::new(String::from("g1h3"), 500);
        game = game.play_move(my_move).unwrap();
        assert_eq!(game.compute_white_elapsed_time(), 1480);
        assert_eq!(game.compute_black_elapsed_time(), 990);
    }
}
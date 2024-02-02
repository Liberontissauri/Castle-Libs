use std::{
    io::{Error, ErrorKind},
    time::SystemTime,
    u16,
};

use pleco::{BitMove, Board};
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};

pub struct ChessGame {
    initial_board: Board,
    moves: Vec<Move>,
    start_time: u32, // in milliseconds
    time_limit: u32, // in milliseconds
    increment: u32,  // in milliseconds
}
impl Serialize for ChessGame {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ChessGame", 5)?;
        state.serialize_field("initial_board", &self.initial_board.fen())?;
        state.serialize_field("moves", &self.moves)?;
        state.serialize_field("start_time", &self.start_time)?;
        state.serialize_field("time_limit", &self.time_limit)?;
        state.serialize_field("increment", &self.increment)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for ChessGame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Initial_Board,
            Moves,
            Start_Time,
            Time_Limit,
            Increment,
        }

        struct ChessGameVisitor;
        impl<'de> serde::de::Visitor<'de> for ChessGameVisitor {
            type Value = ChessGame;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ChessGame")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChessGame, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut initial_board_string = None;
                let mut moves = None;
                let mut start_time = None;
                let mut time_limit = None;
                let mut increment = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Initial_Board => {
                            if initial_board_string.is_some() {
                                return Err(de::Error::duplicate_field("initial_board"));
                            }
                            initial_board_string = Some(map.next_value()?);
                        }
                        Field::Moves => {
                            if moves.is_some() {
                                return Err(de::Error::duplicate_field("initial_board"));
                            }
                            moves = Some(map.next_value()?);
                        }
                        Field::Start_Time => {
                            if start_time.is_some() {
                                return Err(de::Error::duplicate_field("initial_board"));
                            }
                            start_time = Some(map.next_value()?);
                        }
                        Field::Time_Limit => {
                            if time_limit.is_some() {
                                return Err(de::Error::duplicate_field("initial_board"));
                            }
                            time_limit = Some(map.next_value()?);
                        }
                        Field::Increment => {
                            if increment.is_some() {
                                return Err(de::Error::duplicate_field("initial_board"));
                            }
                            increment = Some(map.next_value()?);
                        }
                    }
                }
                let initial_board_string = initial_board_string
                    .ok_or_else(|| de::Error::missing_field("initial_board"))?;
                let moves = moves.ok_or_else(|| de::Error::missing_field("initial_board"))?;
                let increment =
                    increment.ok_or_else(|| de::Error::missing_field("initial_board"))?;
                let start_time =
                    start_time.ok_or_else(|| de::Error::missing_field("initial_board"))?;
                let time_limit =
                    time_limit.ok_or_else(|| de::Error::missing_field("initial_board"))?;

                let initial_board =
                    Board::from_fen(initial_board_string).expect("invalid fen provided");

                Ok(ChessGame {
                    initial_board,
                    moves,
                    increment,
                    start_time,
                    time_limit,
                })
            }
            fn visit_seq<V>(self, mut seq: V) -> Result<ChessGame, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let inital_board: String = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let moves: Vec<Move> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let start_time: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let time_limit: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                let increment: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
                Ok(ChessGame {
                    initial_board: Board::from_fen(&inital_board).unwrap(),
                    moves,
                    start_time,
                    time_limit,
                    increment,
                })
            }
        }
        const FIELDS: &'static [&'static str] = &[
            "initial_board",
            "moves",
            "start_time",
            "time_limit",
            "increment",
        ];
        deserializer.deserialize_struct("ChessGame", FIELDS, ChessGameVisitor)
    }
}
impl ChessGame {
    pub fn compute_current_board(&self) -> Board {
        let mut board = self.initial_board.clone();
        for mov in self.moves.iter() {
            board.apply_uci_move(&mov.uci_move); //Assumes all the previous moves were valid
        }
        board
    }
    pub fn compute_board_at_turn(&self, target_turn: u16) -> Board {
        let mut board = self.initial_board.clone();
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
    pub fn play_move(mut self, mov: Move) -> Result<ChessGame, Error> {
        if self.is_move_legal(&mov) {
            self.moves.push(mov);
            Ok(self)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Tried playing an illegal move",
            ))
        }
    }
    pub fn undo_move(mut self) -> Result<ChessGame, Error> {
        if let Some(mov) = self.moves.pop() {
            let mut board = self.compute_current_board();
            board.undo_move();
            Ok(self)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Tried undoing a move when there are no moves to undo",
            ))
        }
    }
    ///Gives time taken by all white moves without increment
    pub fn compute_white_moves_pure_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() {
            turn += 1;
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::Black {
                elapsed_time += mov.time_taken;
            }
        }
        elapsed_time
    }
    ///Gives time taken by all black moves without increment
    pub fn compute_black_moves_pure_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() {
            turn += 1;
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::Black {
                elapsed_time += mov.time_taken;
            }
        }
        elapsed_time
    }
    pub fn compute_white_moves_time_with_increment(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() {
            turn += 1;
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::Black {
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
    pub fn compute_black_moves_time_with_increment(&self) -> u32 {
        let mut elapsed_time = 0;
        for (mut turn, mov) in self.moves.iter().enumerate() {
            turn += 1;
            let turn_board = self.compute_board_at_turn(turn as u16);
            if turn_board.turn() == pleco::Player::White {
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

    pub fn compute_total_moves_pure_time(&self) -> u32 {
        let mut elapsed_time = 0;
        for mov in self.moves.iter() {
            elapsed_time += mov.time_taken;
        }
        elapsed_time
    }
    pub fn compute_total_move_time_with_increment(&self) -> u32 {
        let mut elapsed_time = 0;
        for mov in self.moves.iter() {
            elapsed_time += mov.time_taken;
            if elapsed_time >= self.increment {
                elapsed_time -= self.increment;
            } else {
                elapsed_time = 0;
            }
        }
        elapsed_time
    }
    /// Returns the time that has been used for the current move
    pub fn compute_current_move_time(&self) -> u32 {
        let mut time_since_first_move = self.compute_total_moves_pure_time();
        let now = SystemTime::now();
        let current_time = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u32;
        current_time - (self.start_time + time_since_first_move)
    }
    pub fn compute_total_elapsed_time(&self) -> u32 {
        let black_time = self.compute_black_moves_pure_time();
        let white_time = self.compute_white_moves_pure_time();
        let current_move_time = self.compute_current_move_time();

        black_time + white_time + current_move_time
    }
    /// Returns the time that has been used by the white player FROM THEIR CLOCK TIME
    pub fn compute_white_used_time(&self) -> u32 {
        let white_moves_time = self.compute_white_moves_time_with_increment();
        let current_move_time = self.compute_current_move_time();
        let turn = self.compute_current_board().turn();
        match turn {
            pleco::Player::White => white_moves_time + current_move_time,
            pleco::Player::Black => white_moves_time,
        }
    }
    /// Returns the time that has been used by the black player FROM THEIR CLOCK TIME
    pub fn compute_black_used_time(&self) -> u32 {
        let black_moves_time = self.compute_black_moves_time_with_increment();
        let current_move_time = self.compute_current_move_time();
        println!("{} e {}", black_moves_time, current_move_time);
        let turn = self.compute_current_board().turn();
        match turn {
            pleco::Player::White => black_moves_time,
            pleco::Player::Black => black_moves_time + current_move_time,
        }
    }

    pub fn is_white_time_over(&self) -> bool {
        let elapsed_time = self.compute_white_used_time();
        elapsed_time > self.time_limit
    }
    pub fn is_black_time_over(&self) -> bool {
        let elapsed_time = self.compute_black_used_time();
        elapsed_time > self.time_limit
    }
    pub fn is_checkmate(&self) -> bool {
        let board = self.compute_current_board();
        board.checkmate()
    }
}

pub struct ChessGameBuilder {
    initial_board: Board,
    moves: Vec<Move>,
    time_limit: u32, // in milliseconds
    increment: u32,  // in milliseconds
}
// Get Time since epoch in miliseconds
// let now = SystemTime::now();
// let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
// println!("Time since the epoch: {:?}", since_the_epoch);
impl ChessGameBuilder {
    pub fn new() -> ChessGameBuilder {
        ChessGameBuilder {
            initial_board: Board::start_pos(),
            moves: Vec::new(),
            time_limit: 0,
            increment: 0,
        }
    }
    pub fn with_initial_board(mut self, board: Board) -> ChessGameBuilder {
        self.initial_board = board;
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
            initial_board: self.initial_board,
            moves: self.moves,
            start_time: now
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u32,
            time_limit: self.time_limit,
            increment: self.increment,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
    use crate::chess::*;
    use pleco::Board;

    #[test]
    fn single_move_time_used() {
        let mut game = ChessGameBuilder::new()
            .with_time_limit(1000 * 60 * 3)
            .with_increment(10)
            .build();

        let my_move = Move::new(String::from("e2e4"), 1000);
        game = game.play_move(my_move).unwrap();
        game.start_time -= 1000;
        assert_eq!(game.compute_white_used_time(), 990);
        assert_eq!(game.compute_black_used_time(), 0);
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
        game.start_time -= 2500;
        assert_eq!(game.compute_white_used_time(), 1480);
        assert_eq!(game.compute_black_used_time(), 990);
    }
}

extern crate game_of_life;
use game_of_life::{GameBoard, LifeCell, NextState};

use std::thread;
use std::time::Duration;

fn main() {
    let mut board = GameBoard::new(170, 60);

    board.set(80, 25, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(81, 25, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(81, 23, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(83, 24, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(84, 25, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(85, 25, LifeCell::Alive(NextState::Unknown)).unwrap();
    board.set(86, 25, LifeCell::Alive(NextState::Unknown)).unwrap();

    loop {
        print!("{}[3J{}", 27 as char, board);

        board.ready();
        board.step();

        thread::sleep(Duration::from_millis(70));
    }
}

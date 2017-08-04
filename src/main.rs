extern crate game_of_life;
use game_of_life::GameBoard;

use std::env;
use std::fs::File;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const FRAME_DELAY: u64 = 70;

fn main() {
    let mut file = File::open(&env::args().skip(1).next()
            .expect("Please provide a filename as a first argument"))
        .expect("Couldn't find file");

    let mut board = GameBoard::from_file(&mut file).unwrap();

    let (tx, rx) = mpsc::sync_channel(1000);

    thread::spawn(move || {
        loop {
            if let Err(_) = tx.send(NextStep) { return; }

            thread::sleep(Duration::from_millis(FRAME_DELAY));
        }
    });

    for _ in rx {
        board.ready();

        print!("{}[2J{}", 27 as char, board);
        
        board.step();
        
    }
}

struct NextStep;

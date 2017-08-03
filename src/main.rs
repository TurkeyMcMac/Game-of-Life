use std::cell::Cell;
use std::fmt;
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

#[derive(Clone, Copy)]
enum NextState {
    Alive, Dead, Unknown
}

impl NextState {
    fn realize(&self) -> Result<LifeCell, StateChangeError> {
        match self {
            &NextState::Alive   => Ok(LifeCell::Alive(NextState::Unknown)),
            &NextState::Dead    => Ok(LifeCell::Dead(NextState::Unknown)),
            &NextState::Unknown => Err(StateChangeError),
        }
    }
}

#[derive(Clone, Copy)]
enum LifeCell {
    Alive(NextState), Dead(NextState)
}

impl LifeCell {
    fn count(&self) -> u32 {
        match self {
            &LifeCell::Alive(_) => 1,
            &LifeCell::Dead(_)  => 0,
        }
    }

    fn ready(&self, neighbors: u32) -> LifeCell {
        let next = match self {
            &LifeCell::Alive(_) if neighbors < 2 || neighbors > 3 => NextState::Dead,
            &LifeCell::Alive(_) => NextState::Alive,
            &LifeCell::Dead(_) if neighbors == 3 => NextState::Alive,
            &LifeCell::Dead(_) => NextState::Dead,
        };

        match self {
            &LifeCell::Alive(_) => LifeCell::Alive(next),
            &LifeCell::Dead(_)  => LifeCell::Dead(next),
        }
    }

    fn step(&self) -> Result<LifeCell, StateChangeError> {
        match self {
            &LifeCell::Alive(into) => into.realize(),
            &LifeCell::Dead(into)  => into.realize(),
        }
    }
}

use std::error::Error;

#[derive(Clone, Copy, Debug)]
struct StateChangeError;

impl Error for StateChangeError {
    fn description(&self) -> &str { "Cannot change cell to state Unknown" }
}

impl fmt::Display for StateChangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

struct GameBoard {
    width: usize,
    height: usize,
    tiles: Vec<Cell<LifeCell>>,
}

impl GameBoard {
    fn new(width: usize, height: usize) -> GameBoard {
        let tile_number = width * height;

        let mut tiles = Vec::with_capacity(tile_number);

        for _ in 0..tile_number {
            tiles.push(Cell::new(LifeCell::Dead(NextState::Unknown)));
        }
        
        GameBoard { width, height, tiles }
    }

    fn get(&self, x: usize, y: usize) -> Option<LifeCell> {
        Some(match self.tiles.get(y * self.width + x) {
            Some(v) => v,
            None    => return None,
        }.get())
    }


    fn set(&self, x: usize, y: usize, value: LifeCell) -> Result<(), &str> {
        match self.tiles.get(y * self.width + x) {
            Some(v) => v,
            None    => return Err("Coordinates out of bounds"),
        }.set(value);

        Ok(())
    }

    fn count_neighbors(&self, index: usize) -> u32 {
        let (x, y) = self.coords(index);

        let up = (y + 1) % self.height;
        let down = (self.height + y - 1) % self.height;
        let right = (x + 1) % self.width;
        let left = (self.width + x - 1) % self.width;
/*
        println!(r#"Counting neighbors of ({}, {}).
up: {},
down: {},
right: {},
left: {},
                 "#, x, y, up, down, right, left);*/
        self.get(x,     up  ).unwrap().count() +
        self.get(right, up  ).unwrap().count() +
        self.get(right, y   ).unwrap().count() +
        self.get(right, down).unwrap().count() +
        self.get(x,     down).unwrap().count() +
        self.get(left,  down).unwrap().count() +
        self.get(left,  y   ).unwrap().count() +
        self.get(left,  up  ).unwrap().count()
    }

    fn coords(&self, index: usize) -> (usize, usize) {
        let mut x = index;
        let mut y = 0;

        while x > self.width - 1 {
            x -= self.width;
            y += 1;
        }

        (x, y)
    }

    fn ready(&mut self) {
        for (i, cell) in self.tiles.iter().enumerate() {
            cell.set(cell.get().ready(self.count_neighbors(i)));
        }
    }

    fn step(&mut self) {
        for cell in self.tiles.iter() {
            cell.set(cell.get().step().unwrap());
        }
    }
}

impl fmt::Display for GameBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            self.tiles.iter().enumerate()
                .map(|(i, tile)| {
                    let mut icon = String::from(match tile.get() {
                        LifeCell::Alive(_) => "#",
                        LifeCell::Dead(_)  => "_",
                    });

                    if (i + 1) % self.width == 0 {
                        icon += "\n";
                    }

                    icon
                })
                .collect::<String>()
        )
    }
}

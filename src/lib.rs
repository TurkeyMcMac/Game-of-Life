use std::cell::Cell;
use std::fmt;
use std::io;
use std::io::{BufRead, BufReader};
use std::fs::File;

const ALIVE: &str = "██";
const DEAD: &str = "  ";
const GROWING: &str = "░░";
const DYING: &str = "▓▓";

const READ_ALIVE: u8 = b'=';
const READ_DEAD: u8 = b'-';

#[derive(Clone, Copy)]
pub enum NextState {
    Alive, Dead, Unknown
}

impl NextState {
    fn realize(&self) -> Result<LifeCell, StateChangeError> {
        match *self {
            NextState::Alive   => Ok(LifeCell::Alive(NextState::Unknown)),
            NextState::Dead    => Ok(LifeCell::Dead(NextState::Unknown)),
            NextState::Unknown => Err(StateChangeError),
        }
    }
}

#[derive(Clone, Copy)]
pub enum LifeCell {
    Alive(NextState), Dead(NextState)
}

impl LifeCell {
    fn count(&self) -> u32 {
        match *self {
            LifeCell::Alive(_) => 1,
            LifeCell::Dead(_)  => 0,
        }
    }

    fn decide(&self, neighbors: u32) -> NextState {
       match *self {
            LifeCell::Alive(_) if neighbors == 2 || neighbors == 3 => NextState::Alive,
            LifeCell::Dead(_) if neighbors == 3 => NextState::Alive,
            _ => NextState::Dead,
        }
    }

    fn ready(&self, neighbors: u32) -> LifeCell {
        let next = self.decide(neighbors);

        match *self {
            LifeCell::Alive(_) => LifeCell::Alive(next),
            LifeCell::Dead(_)  => LifeCell::Dead(next),
        }
    }

    fn step(&self) -> Result<LifeCell, StateChangeError> {
        match *self {
            LifeCell::Alive(into) => into.realize(),
            LifeCell::Dead(into)  => into.realize(),
        }
    }
}

impl fmt::Display for LifeCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            LifeCell::Alive(NextState::Alive) => ALIVE,
            LifeCell::Alive(NextState::Dead)  => DYING,
            LifeCell::Dead(NextState::Alive)  => GROWING,
            LifeCell::Dead(NextState::Dead)   => DEAD,
            _ => "?",
        })
    }
}

use std::error::Error;

#[derive(Clone, Copy, Debug)]
pub struct StateChangeError;

impl Error for StateChangeError {
    fn description(&self) -> &str { "Cannot change cell to state Unknown" }
}

impl fmt::Display for StateChangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub struct GameBoard {
    width: usize,
    height: usize,
    tiles: Vec<Cell<LifeCell>>,
}

impl GameBoard {
    pub fn new(width: usize, height: usize) -> GameBoard {
        let tile_number = width * height;

        let mut tiles = Vec::with_capacity(tile_number);

        for _ in 0..tile_number {
            tiles.push(Cell::new(LifeCell::Dead(NextState::Unknown)));
        }
        
        GameBoard { width, height, tiles }
    }

    pub fn from_file(file: &mut File) -> io::Result<GameBoard> {
        let mut reader = BufReader::new(file);

        let mut first_line = String::new();

        reader.read_line(&mut first_line)?;

        let mut parameters = first_line.split_whitespace()
            .map(|num_string| if let Ok(v) = num_string.parse::<usize>() { v } else { 0 });

        let width = parameters.next().unwrap_or(0);
        let height = parameters.next().unwrap_or(0);

        let tiles = reader.lines()
            .flat_map(|line| {
                let line = line.expect("Problem reading line");
                assert!(line.len() == width, "line length {} != width {}", line.len(), width);
                
                line.into_bytes().into_iter()
                    .filter_map(|byte| match byte {
                        READ_ALIVE => Some(Cell::new(LifeCell::Alive(NextState::Unknown))),
                        READ_DEAD  => Some(Cell::new(LifeCell::Dead(NextState::Unknown))),
                        _          => None,
                    })
            })
            .collect::<Vec<Cell<LifeCell>>>();

        assert!(tiles.len() == width * height);
        Ok(GameBoard { width, height, tiles })
    }

    pub fn get(&self, x: usize, y: usize) -> Option<LifeCell> {
        Some(match self.tiles.get(y * self.width + x) {
            Some(v) => v,
            None    => return None,
        }.get())
    }


    pub fn set(&self, x: usize, y: usize, value: LifeCell) -> Result<(), &str> {
        match self.tiles.get(y * self.width + x) {
            Some(v) => v,
            None    => return Err("Coordinates out of bounds"),
        }.set(value);

        Ok(())
    }

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> LifeCell {
        self.tiles.get_unchecked(y * self.width + x).get()
    }


    fn count_neighbors(&self, index: usize) -> u32 {
        let (x, y) = self.coords(index);

        let up = (y + 1) % self.height;
        let down = (self.height + y - 1) % self.height;
        let right = (x + 1) % self.width;
        let left = (self.width + x - 1) % self.width;
        
        unsafe {
            self.get_unchecked(x,     up  ).count() +
            self.get_unchecked(right, up  ).count() +
            self.get_unchecked(right, y   ).count() +
            self.get_unchecked(right, down).count() +
            self.get_unchecked(x,     down).count() +
            self.get_unchecked(left,  down).count() +
            self.get_unchecked(left,  y   ).count() +
            self.get_unchecked(left,  up  ).count()
        }
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

    pub fn ready(&mut self) {
        for (i, cell) in self.tiles.iter().enumerate() {
            cell.set(cell.get().ready(self.count_neighbors(i)));
        }
    }

    pub fn step(&mut self) {
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
                    let mut icon = tile.get().to_string();

                    if (i + 1) % self.width == 0 {
                        icon += "\n";
                    }

                    icon
                })
                .collect::<String>()
        )
    }
}


use rand::Rng;
use crate::ws_3d::grid::{Grid3D,Face};

#[derive(Copy,Clone)]
pub enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

impl Direction {
    pub fn to_vector(&self) -> (i32,i32) {
        match self {
            Direction::N => (0,-1),
            Direction::E => (1,0),
            Direction::S => (0,1),
            Direction::W => (-1,0),
            Direction::NE => (1,-1),
            Direction::SE => (1,1),
            Direction::SW => (-1,1),
            Direction::NW => (-1,-1),
        }
    }
    pub fn to_symbol(d : (i32,i32)) -> Direction {
        match d {
            (0,-1) => Direction::N,
            (1,0) => Direction::E,
            (0,1) => Direction::S,
            (-1,0) => Direction::W,
            (1,-1) => Direction::NE,
            (1,1) => Direction::SE,
            (-1,1) => Direction::SW,
            (-1,-1) => Direction::NW,
            _ => panic!("Invalid direction vector provided")
        }
    }
}

pub const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub const EMPTY : u8 = 0;
pub const DIRECTIONS : [(i32,i32);8] = [
    (0,-1),
    (1,0),
    (0,1),
    (-1,0),
    (1,-1),
    (1,1),
    (-1,1),
    (-1,-1)
];

pub fn get_cell(grid : &Vec<Vec<u8>>, x : i32, y : i32, d : (i32,i32), pos : i32) -> Option<u8> {
    let px : i32 = x + d.0 * pos;
    let py : i32 = y + d.1 * pos;
    let dim : i32 = grid.len() as i32;
    if px < 0 || py < 0 || px >= dim || py >= dim {
        return None;
    }
    Some(grid[px as usize][py as usize])
}

// need to check that these two conditions are both false for validation:
// (1) word or "reverse" word is a substring of a, where a is a word in "used"
// (2) b or "reverse" b is a substring of word, where b is a word in "used"
pub fn validate_word(word : &str, words_used : &Vec<String>) -> bool {
    let rev_word : String = word.chars().rev().collect::<String>();
    !words_used.iter().any(|w:&String | {
        let rev_w: String = w.chars().rev().collect::<String>();
        word.contains(w) || w.contains(&word) || word.contains(&rev_w) || w.contains(&rev_word)
    })
}

pub fn get_random_letter_byte() -> u8 {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let pos: usize = rng.random_range(0..26);
    LETTERS.as_bytes()[pos]
}

pub fn read_wordlist() -> Result<Vec<String>, std::io::Error> {
    // Parse wordlist as line separated vector, handle errors appropriately
    let contents: &'static str = include_str!("wordlist.txt");
    let words: Vec<String> = contents.lines().map(|s: &str| s.to_owned()).collect();
    Ok(words)
}

/* 3D puzzle Util */

pub fn get_max_dist(grid : &Grid3D, face : Face, x : i32, y : i32, d : (i32,i32)) -> i32 {
    let mut dist : i32 = 0;
    // compute distance from boundary for x and y (account for multiple planes)
    // minimum between these distances represents the maximum displacement
    loop {
        if get_pos_3d(grid, face, x, y, d, dist).is_some() {
            dist+=1;
        } else {
            break;
        }
    }
    dist
} 

pub fn get_pos_3d(grid : &Grid3D, face : Face, x : i32, y : i32, d : (i32,i32), pos : i32) -> Option<(Face,i32,i32,(i32,i32))> {
    let dim : i32 = grid.get_size() as i32;
    let mut px : i32 = x;
    let mut py : i32 = y;
    let mut pf : Face = face;
    let mut pd : (i32,i32) = d;
    let mut dist = pos;
    while dist > 0 {
        // check up, down, left, right
        // find the minimum distance needed to exit the current face 
        let mut min_dist : i32 = i32::MAX;
        // UP
        if pd.0 == -1 { 
            min_dist = std::cmp::min(min_dist, px+1);
        }
        // DOWN
        if pd.0 == 1 { 
            min_dist = std::cmp::min(min_dist, dim-px); // (n-1-x)+1=n-x
        }
        // LEFT
        if pd.1 == -1 { 
            min_dist = std::cmp::min(min_dist, py+1);
        }
        // RIGHT
        if pd.1 == 1 {
            min_dist = std::cmp::min(min_dist, dim-py); // (n-1-y)+1=n-y
        }
        let delta : i32 = std::cmp::min(min_dist, dist);
        px += delta * pd.0;
        py += delta * pd.1;
        dist -= delta;
        // Check if x and y are both out of bounds
        if (px < 0 || px >= dim) && (py < 0 || py >= dim) {
            return None;
        }
        // Check if the current face boundary has been found
        // Determine the updated face on a case-by-case analysis
        // Note that x increases from top to bottom, and y increases from left to right
        if px < 0 || py < 0 || px >= dim || py >= dim {
            match pf {
                Face::Left => {
                    /*
                        +---+
                        | T |
                        +---+
                        +---+---+
                        | L | R |
                        +---+---+
                    */
                    // Check if position is invalid
                    if px >= dim || py < 0 {
                        return None;
                    }
                    // Position is valid, so determine updated face
                    if px < 0 {
                        px += dim;
                        pf = Face::Top;
                    } else if py >= dim {
                        py -= dim;
                        pf = Face::Right;
                    }
                },
                Face::Right => {
                    /*
                            +---+
                            | T |
                            +---+
                        +---+---+
                        | L | R |
                        +---+---+
                    */
                    if px >= dim || py >= dim {
                        return None;
                    }
                    // Position is valid, so determine updated face
                    if px < 0 {
                        let v : i32 = px+dim;
                        px = dim-1-py;
                        py = v;
                        pf = Face::Top;
                        // (dx,dy) => (-dy,dx)
                        pd = (-pd.1, pd.0);
                    } else if py < 0 {
                        py += dim;
                        pf = Face::Left;
                    }
                },
                Face::Top => {
                    /*
                        +---+---+
                        | T | R |
                        +---+---+
                        +---+
                        | L |
                        +---+
                    */
                    if px < 0 || py < 0 {
                        return None;
                    }
                    if px >= dim {
                        px -= dim;
                        pf = Face::Left;
                    } else if py >= dim {
                        let v : i32 = py-dim;
                        py = dim-1-px;
                        px = v;
                        pf = Face::Right;
                        // (dx,dy) => (dy,-dx)
                        pd = (pd.1, -pd.0);
                    }
                },
            }
        }
    }
    Some((pf,px,py,pd))
}

// Old implementation of get_cell_3d (doesn't use math so it's slower)
pub fn _get_pos_3d(grid : &Grid3D, face : Face, x : i32, y : i32, d : (i32,i32), pos : i32) -> Option<(Face,i32,i32,(i32,i32))> {
    let dim : i32 = grid.get_size() as i32;
    let mut px : i32 = x;
    let mut py : i32 = y;
    let mut pf : Face = face;
    let mut pd : (i32,i32) = d;
    for _ in 1..=pos {
        px += pd.0;
        py += pd.1;
        // Check if x and y are both out of bounds
        if (px < 0 || px >= dim) && (py < 0 || py >= dim) {
            return None;
        } 
        // Check if the current face boundary has been found
        // Determine the updated face on a case-by-case analysis
        // Note that x increases from top to bottom, and y increases from left to right
        if px < 0 || py < 0 || px >= dim || py >= dim {
            match pf {
                Face::Left => {
                    /*
                        +---+
                        | T |
                        +---+
                        +---+---+
                        | L | R |
                        +---+---+
                    */
                    // Check if position is invalid
                    if px >= dim || py < 0 {
                        return None;
                    }
                    // Position is valid, so determine updated face
                    if px < 0 {
                        px += dim;
                        pf = Face::Top;
                    } else if py >= dim {
                        py -= dim;
                        pf = Face::Right;
                    }
                },
                Face::Right => {
                    /*
                            +---+
                            | T |
                            +---+
                        +---+---+
                        | L | R |
                        +---+---+
                    */
                    if px >= dim || py >= dim {
                        return None;
                    }
                    // Position is valid, so determine updated face
                    if px < 0 {
                        let v : i32 = px+dim;
                        px = dim-1-py;
                        py = v;
                        pf = Face::Top;
                        // (dx,dy) => (-dy,dx)
                        pd = (-pd.1, pd.0);
                    } else if py < 0 {
                        py += dim;
                        pf = Face::Left;
                    }
                },
                Face::Top => {
                    /*
                        +---+---+
                        | T | R |
                        +---+---+
                        +---+
                        | L |
                        +---+
                    */
                    if px < 0 || py < 0 {
                        return None;
                    }
                    if px >= dim {
                        px -= dim;
                        pf = Face::Left;
                    } else if py >= dim {
                        let v : i32 = py-dim;
                        py = dim-1-px;
                        px = v;
                        pf = Face::Right;
                        // (dx,dy) => (dy,-dx)
                        pd = (pd.1, -pd.0);
                    }
                },
            }
        }
    }
    Some((pf,px,py,pd))
}

pub fn get_cell_3d(grid : &Grid3D, face : Face, x : i32, y : i32, d : (i32,i32), pos : i32) -> Option<u8> {
    if let Some((pf, px, py, _)) = get_pos_3d(grid, face, x, y, d, pos) {
        let plane : &Vec<Vec<u8>> = grid.get_face(pf);
        Some(plane[px as usize][py as usize])
    } else {
        None
    }
}
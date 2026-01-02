use rand::Rng;
use core::panic;
use std::collections::{HashSet,HashMap};
use crate::ws_2d::trie::Trie;
use crate::common::util::{get_random_letter_byte,read_wordlist,Direction,DIRECTIONS,EMPTY};

const FAIL_BOUND : i32 = 1_000_000;
const RETRY_BOUND : i32 = 1_000;
const MIN_WORD_LEN : i32 = 5;

pub struct WordSearch {
    pub grid : Vec<Vec<u8>>,
    pub words : Vec<String>
}

pub fn init() -> Trie {
    // build trie
    let mut trie  = Trie::new();
    // load the wordlist
    let words: Vec<String> = match read_wordlist() {
        Ok(file) => file,
        Err(e) => panic!("Error occurred when reading wordlist: {}", e),
    };
    trie.load(&words);
    trie
}

// Places the word in the grid (assumes that the placement is valid)
fn place_word (grid : &mut Vec<Vec<u8>>, x : i32, y : i32, d : (i32,i32), word : &String, dir_map : &mut HashMap<(i32,i32),Vec<Direction>>, inter_freq : &mut [HashSet<(i32,i32)>; 4], word_pos : &mut HashMap<String,(i32,i32)>, spaces_used : &mut i32) {
    let dim: usize = word.len();
    let n : i32 = dim as i32;
    let ds : Direction = Direction::to_symbol(d);
    word_pos.insert(word.to_owned(), (x,y));
    let word_bytes: &[u8] = word.as_bytes();
    for i in 0..n {
        let px : i32 = x + d.0 * i;
        let py : i32 = y + d.1 * i;
        let pos : (i32,i32) = (px,py);
        // Update grid character if space is empty
        if grid[px as usize][py as usize] == EMPTY {
            grid[px as usize][py as usize] = word_bytes[i as usize];
            dir_map.insert(pos, vec![ds]);
            *spaces_used+=1;
            inter_freq[1].insert(pos);
        } else {
            if let Some(v) = dir_map.get_mut(&pos) {
                // Add additional direction to the current position
                v.push(ds);
                if v.len()-1 >= inter_freq.len() {
                    continue;
                }
                // Adjust position in inter_freq accordingly
                inter_freq[v.len()-1].remove(&pos);
                if v.len() < inter_freq.len() {
                    inter_freq[v.len()].insert(pos);
                }
            } else {
                panic!("Failed to retrieve direction for cell {},{}", x, y);
            }
        }
    }
}
// Select a random "valid" (unused) direction, based on the current starting position
fn get_random_valid_direction (grid : &Vec<Vec<u8>>, x : i32, y : i32, banned_dir : &Vec<(i32,i32)>) -> Option<((i32,i32),i32)> {
    // Exit early if no valid direction exists
    if banned_dir.len() >= 4 {
        return None;
    }
    let n : i32 = grid.len() as i32;
    // Prune invalid directions before selecting a random direction
    // Simply skip direction if it or its opposite is banned
    let directions : Vec<(i32,i32)> = DIRECTIONS.iter().filter(|dir: &&(i32, i32)| {
        !banned_dir.iter().any(|banned : &(i32,i32)| {
            (dir.0 == banned.0 && dir.1 == banned.1) || (-dir.0 == banned.0 && -dir.1 == banned.1)
        })
    }).copied().collect();
    let cnt : usize = directions.len();
    if cnt == 0 {
        return None;
    }
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let r: usize = rng.random_range(0..cnt);
    let d: (i32, i32) = directions[r];
    // compute distance from boundary for x and y
    // minimum between these distances represents the maximum word length
    let x_dist : i32 = if d.0 > 0 { n-x } else if d.0 < 0 { x+1 } else { i32::MAX };
    let y_dist : i32 = if d.1 > 0 { n-y } else if d.1 < 0 { y+1 } else { i32::MAX };
    Some((d, std::cmp::min(x_dist, y_dist)))
}
// Generate a random "valid" starting position. Returns (x,y,direction_vector,min_len,max_len) 
fn get_random_starting_state (grid : &Vec<Vec<u8>>, dir_map : &HashMap<(i32,i32),Vec<Direction>>, inter_freq : &[HashSet<(i32,i32)>; 4]) -> Option<(i32,i32,(i32,i32),i32,i32)> {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    // first perform the weighted randomization
    // 55-35-10 selection between 1,2,3 (groups a, b, c respectively)
    // gcd(55,35,10) = 5 -> 11-7-2
    let total : i32 = 20; // 11+7+2=20
    // randomly select non-empty group (repeatedly do this until you get one)
    let mut r : i32;
    let mut group : usize;
    loop {
        r = rng.random_range(0..total);
        group = if r < 11 { 1 } else if r < 11+7 { 2 } else { 3 };
        if inter_freq[group].len() != 0 {
            break;
        } 
    }
    let group_set: &HashSet<(i32, i32)> = &inter_freq[group];
    let r: usize = rng.random_range(0..group_set.len());
    let rpos: &(i32, i32) = group_set.iter().nth(r).unwrap();
    let x : i32 = rpos.0;
    let y : i32 = rpos.1;
    // after starting position (x,y) has been selected, choose a random unused direction
    let banned_dir: &Vec<(i32, i32)> = match dir_map.get(rpos) {
        None => panic!("Unable to access directions for cell {},{}", x, y),
        Some(v) => &v.iter().map(|d: &Direction| Direction::to_vector(d)).collect()
    };
    let dim: usize  = grid.len();
    let n: i32 = dim as i32;
    let (d,max_len) = get_random_valid_direction(grid, x, y, banned_dir)?;
    // 50/50 chance for the flipped direction being used
    let flip_dir = rng.random_bool(0.5);
    let rev_max_len : i32 = std::cmp::min(
        if -d.0 > 0 { n-x } else if -d.0 < 0 { x+1 } else { i32::MAX },
        if -d.1 > 0 { n-y } else if -d.1 < 0 { y+1 } else { i32::MAX }
    );
    if flip_dir {
        // find offset in the original direciton
        // place word in opposite (flipped) direction
        let delta : i32 = rng.random_range(0..max_len);
        // compute starting position
        let px : i32 = x + d.0 * delta;
        let py : i32 = y + d.1 * delta;
        // reverse the direction (word should "pass over" randomly selected cell, (x,y), going in the opposite direction)
        // min length is delta+1 (so that randomly selected cell is included)
        // max length is (delta+1)+(maxLenRev)-1 = delta+maxLenRev (respects boundary for opposite direction)
        return Some((px, py, (-d.0,-d.1), delta+1, delta+rev_max_len));
    }
    // find offset in the opposite direction
    // place word in the original direction
    let delta : i32 = rng.random_range(0..rev_max_len);
    // compute starting position
    let px : i32 = x + d.0 * -delta;
    let py : i32 = y + d.1 * -delta;
    // min length is delta+1 (so that randomly selected cell is included)
    // max length is (delta+1)+(maxLen)-1 = delta+maxLen (respects boundary for original direction)
    Some((px, py, d, delta+1, delta+max_len))
}

fn generate_puzzle_word (trie : &Trie, grid : &mut Vec<Vec<u8>>, x : i32, y : i32, mut d : Option<(i32,i32)>, mut min_len : Option<i32>, mut max_len : Option<i32>, words_used : &mut Vec<String>, dir_map : &mut HashMap<(i32,i32),Vec<Direction>>, inter_freq : &mut [HashSet<(i32,i32)>; 4], word_pos : &mut HashMap<String,(i32,i32)>, spaces_used : &mut i32) -> bool {
    // Check if we are in the starting word case
    if d.is_none() && max_len.is_none() && min_len.is_none() {
        match get_random_valid_direction(grid, x, y, &vec![]) {
            Some(t) => {
                d = Some(t.0);
                max_len = Some(t.1);
                min_len = Some(0);
            },
            None => return false,
        };
    }
    // Unwrap all option variables
    let d: (i32, i32) = d.unwrap();
    let min_len: i32 = min_len.unwrap();
    let max_len: i32 = max_len.unwrap(); 
    // Attempt to retrieve a word from the trie for the starting position (x,y) and direction d
    if let Some(word)  = trie.get_random(grid, x, y, d, min_len, max_len, words_used) {
        place_word(grid, x, y, d, &word, dir_map, inter_freq, word_pos, spaces_used);
        words_used.push(word);
        true
    } else {
        false
    }
}

fn validate_puzzle (grid : &mut Vec<Vec<u8>>, words_used : &Vec<String>, word_pos : &HashMap<String, (i32, i32)>, dir_map : &HashMap<(i32,i32),Vec<Direction>>) -> bool {
    // build a trie with the current words
    // use trie to find all words that exist in a direction
    let mut words_trie = Trie::new();
    words_trie.load(words_used);
    words_trie.verify_and_repair_grid(grid, words_used, word_pos, dir_map)
}

pub fn generate(dim : usize, trie : &Trie) -> WordSearch {
    build_puzzle(dim, trie, 0)
} 

fn build_puzzle(dim : usize, trie : &Trie, fails : i32) -> WordSearch {
    let n : i32 = dim as i32;
    if n < MIN_WORD_LEN {
        panic!("No words in the trie can fit into the puzzle");
    }
    if fails >= FAIL_BOUND {
        panic!("Could not build puzzle after {} retries", FAIL_BOUND);
    }
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    // Initialize grid
    let mut grid : Vec<Vec<u8>> = vec![vec![EMPTY; dim]; dim];
    // Keep track of the words used
    let mut words_used : Vec<String> = vec![];
    // Mark the direction that was used by (x,y)
    let mut spaces_used = 0;
    let mut dir_map : HashMap<(i32,i32),Vec<Direction>> = HashMap::new();
    // Store the starting position
    let mut word_pos : HashMap<String,(i32,i32)> = HashMap::new();
    // Store sets for the number of words that pass through a cell (either 1,2,3)
    let mut inter_freq : [HashSet<(i32,i32)>; 4] = [
        HashSet::new(), // unused
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    ];
    // Place first word (it should start or cross near the center)
    let bounds : (i32,i32) = (n/4, (3*n+3)/4);
    let range : i32 = bounds.1 - bounds.0;
    let r1 : i32 = rng.random_range(0..range);
    let r2 : i32 = rng.random_range(0..range);
    let px : i32 = bounds.0 + r1;
    let py : i32 = bounds.0 + r2;
    // Keep trying to place the first word (loop is probabilistically necessary for )
    loop {
        if generate_puzzle_word(trie, &mut grid, px, py, None, None, None, &mut words_used, &mut dir_map, &mut inter_freq, &mut word_pos, &mut spaces_used) {
            break;
        }
    }
    // Desired (performance) ratio for spaces used to total spaces is 0.75 or 3/4
    let desired_spaces : i32 = (3*n*n)/4;
    let mut retry_count : i32 = 0;
    // Place words until the performance ratio is satisfied or until all words have been used
    // Latter condition is a sanity check and should not occur unless the word list can be fully exhausted
    while spaces_used < desired_spaces && words_used.len() < trie.get_size() {
        if let Some((x, y, d, min_len, max_len)) = get_random_starting_state(&grid, &dir_map, &inter_freq) {
            // Attempt to generate word
            if generate_puzzle_word(trie, &mut grid, x, y, Some(d), Some(min_len), Some(max_len), &mut words_used, &mut dir_map, &mut inter_freq, &mut word_pos, &mut spaces_used) {
                retry_count = 0;
            } else {
                retry_count+=1;
            }
        } else {
            retry_count+=1;
        }
        if retry_count > RETRY_BOUND {
            break;
        }
    }
    // Fill in the blank spaces
    for i in 0..dim {
        for j in 0..dim {
            if grid[i][j] == EMPTY {
                grid[i][j] = get_random_letter_byte();
            }
        }
    }
    // Valid the puzzle
    let is_puzzle_valid : bool = validate_puzzle(&mut grid, &words_used, &word_pos, &dir_map);
    // If the puzzle cannot be fixed, then the puzzle must be rebuild
    if !is_puzzle_valid {
        return build_puzzle(dim, trie, fails+1);
    }
    WordSearch { grid : grid, words : words_used.iter().map(|w: &String| w.to_string()).collect() }
} 
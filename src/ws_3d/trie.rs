use rand::Rng;
use std::collections::HashMap;
use crate::ws_3d::grid::{Grid3D, Face};
use crate::common::util::{get_random_letter_byte, validate_word, get_cell_3d, get_pos_3d, Direction, DIRECTIONS, EMPTY};

struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    end_of_word: bool,
    min_len: i32,
    max_len: i32,
}

pub struct Trie {
    root: TrieNode,
    size: usize,
    min_len: i32,
    max_len: i32,    
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode {
            children: [const { None }; 26],
            end_of_word: false,
            min_len: i32::MAX,
            max_len: 0,
        }
    }
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode { children: [const { None }; 26], end_of_word: false, min_len: i32::MAX, max_len: 0 },
            size: 0,
            min_len: i32::MAX,
            max_len: 0,
        }
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    // insert word in true (false indicates duplicate)
    pub fn insert(&mut self, word : &str) -> bool {
        let n : i32 = word.len() as i32;
        let mut node: &mut TrieNode = &mut self.root;
        for b in word.bytes() {
            let idx : usize = (b - b'a') as usize;
            if node.children[idx].is_none() {
                let mut t : TrieNode = TrieNode::new();
                t.max_len = n;
                t.min_len = n;
                node.children[idx] = Some(Box::new(t));
            }
            node.min_len = std::cmp::min(node.min_len, n);
            node.max_len = std::cmp::max(node.max_len, n);
            node = node.children[idx].as_mut().unwrap();
        }
        // check if the word exists
        if node.end_of_word {
            return false;
        }
        self.size+=1;
        node.end_of_word = true;
        true
    }
    /*
        Fetch a random word from the Trie
        Grid layout (for each face): (y increases from left to right, x increases from top to bottom)
        +---------y
        |
        |
        |
        |
        x
    */
    pub fn get_random(&self, grid : &Grid3D, face : Face, x : i32, y : i32, d : (i32,i32), min_len : i32, max_len : i32, words_used : &Vec<String>) -> Option<String> {
        const MAX_ITERATIONS : i32 = 32;
        if max_len < self.min_len || min_len > self.max_len {
            return None;
        }
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        for _ in 0..MAX_ITERATIONS {
            let mut words_found : Vec<String> = vec![];
            let mut curr : Vec<u8> = vec![];
            let mut node : &TrieNode = &self.root;
            let mut pos: i32 = 0;
            // search while (1) position has not exceeded maximum length, and (2) maximum word length in subtrie is at least the minimum required length
            // also checking that trie node is valid
            while pos < max_len && node.max_len >= min_len {
                // check if shortest possible word from the current node will not fit (out of bounds)
                if get_cell_3d(grid, face, x, y, d, node.min_len-1).is_none() {
                    break;
                };
                let cell: u8 = match get_cell_3d(grid, face, x, y, d, pos) {
                    Some(c) => c,
                    None => panic!("Invalid grid cell access"),
                };
                // select random valid index for next trie node to process
                let mut idx: usize = usize::MAX;
                // check if cell does not have a set character yet
                if cell == EMPTY {
                    let mut cnt: i32 = 0;
                    for child in &node.children {
                        if !child.is_none() {
                            cnt+=1;
                        }
                    }
                    if cnt == 0 {
                        break;
                    }
                    let r : i32 = rng.random_range(0..cnt)+1;
                    cnt = 0;
                    for i in 0..node.children.len() {
                        let child: &Option<Box<TrieNode>> = &node.children[i];
                        if child.is_none() {
                            continue;
                        }
                        cnt+=1;
                        if cnt == r {
                            idx = i;
                            break;
                        }
                    }
                } else {
                    // cell already has a set character
                    idx = (cell - b'a') as usize;
                    // check if the child is valid
                    if node.children[idx].is_none() {
                        break;
                    }
                }
                if cell == EMPTY {
                    curr.push(b'a' + (idx as u8));
                } else {
                    curr.push(cell);
                }
                if let Some(next_node) = node.children[idx].as_ref() {
                    node = next_node;
                } else {
                    break;
                }
                let word: String = match String::from_utf8(curr.clone()) {
                    Ok(w) => w,
                    Err(err) => panic!("UTF-8 error occurred when getting random word: {}", err),
                };
                if node.end_of_word && validate_word(&word, &words_used) {
                    words_found.push(word);
                }
                pos+=1;
            }
            let n : i32 = words_found.len() as i32;
            // if no words were found, try again
            // if one word was found, return that word
            if n == 0 {
                continue;
            }
            if n == 1 {
                return Some(words_found[0].clone());
            }
            // use weighted randomization (favoring longer words)
            let power: u32 = 3;
            let mut total: u32 = 0;
            // compute total weight
            for i in 0..n {
                total += ((i+2) as u32).pow(power);
            }
            let r: u32 = rng.random_range(0..total);
            let mut curr: u32 = 0;
            for i in 0..n {
                curr += ((i+2) as u32).pow(power);
                if curr > r {
                    return Some(words_found[i as usize].clone());
                }
            }
            panic!("Random selection from words found failed");
        }
        None
    }

    // Populate the trie with the wordlist
    pub fn load(&mut self, words : &Vec<String>) {
        for w in words {
            self.insert(&w);
            self.min_len = std::cmp::min(self.min_len, w.len() as i32);
            self.max_len = std::cmp::max(self.max_len, w.len() as i32);
        }
    }
    // For each cell (x,y) in the grid, consider all possible directions
    // After finding all instances of each word, check the frequencies
    // Remove any duplicates by checking the placement location
    pub fn verify_and_repair_grid(&mut self, grid : &mut Grid3D, words : &Vec<String>, word_pos : &HashMap<String,(Face,i32,i32)>, dir_map : &HashMap<(Face,i32,i32),Vec<Direction>>) -> bool {
        // Cannot guarantee that the grid is correct after succesful repair, so loop is necessary
        for _ in 0..100_000 {
            // map format: { word : [(x,y,direction)] }, direction: (dx,dy)
            let mut freq : HashMap<&str,Vec<(Face,i32,i32,(i32,i32))>> = HashMap::new();
            for w in words {
                freq.insert(w, vec![]);
            }
            let dim : i32  = grid.get_size() as i32;
            for &face in Face::iterator() {
                for x in 0..dim {
                    for y in 0..dim {
                        for d in DIRECTIONS {
                            let mut node: &mut TrieNode = &mut self.root;
                            let mut curr : Vec<u8> = vec![];
                            let mut pos: i32 = 0;
                            loop {
                                if let Some((pf,px,py,_)) = get_pos_3d(grid, face, x, y, d, pos) {
                                    let plane : &Vec<Vec<u8>> = grid.get_face(pf);
                                    let b: u8 = plane[px as usize][py as usize];
                                    let idx: usize = (b - b'a') as usize;
                                    if node.children[idx].is_none() {
                                        break;
                                    }
                                    node = node.children[idx].as_mut().unwrap();
                                    curr.push(b);
                                    // Check if word was found
                                    if node.end_of_word {
                                        let word: &str = match std::str::from_utf8(&curr) {
                                            Ok(w) => w,
                                            Err(err) => panic!("UTF-8 error occurred when scanning grid: {}", err),
                                        };
                                        match freq.get_mut(word) {
                                            Some(v) => v.push((face,x,y,d)),
                                            None => panic!("Error occurred when updating frequency map"),
                                        }
                                        break;
                                    }
                                    pos+=1;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            let mut no_duplicates_found : bool = true;
            let mut repair_failed : bool = false;
            for (&word, occurrences) in &freq {
                if occurrences.is_empty() {
                    panic!("Word ({}) is missing from the grid", word);
                }
                // Check if there are too many occurrences
                if occurrences.len() > 1 {
                    let true_pos: (Face, i32, i32) = match word_pos.get(word) {
                        None => panic!("Word has no position"),
                        Some(&p) => p
                    };
                    no_duplicates_found = false;
                    let mut duplicates_overlap: bool = true;
                    for &(face, x,y, d) in occurrences {
                        // Check if current occurrence matches the word's stored position
                        if true_pos.0 == face && true_pos.1 == x && true_pos.2 == y {
                            continue;
                        }
                        // Stored position not matched, so grid must be verified again
                        // Attempt to make changes where appropriate
                        duplicates_overlap = false;
                        let n: usize = word.len();
                        let mut cell_fix_made: bool = false;
                        for i in 0..n {
                            if let Some((pf, px, py, _)) = get_pos_3d(grid, face, x, y, d, i as i32) {
                                // check if letter belongs to a word
                                if dir_map.contains_key(&(pf,px,py)) {
                                    continue;
                                }
                                let plane : &mut Vec<Vec<u8>> = grid.get_face_mut(pf);
                                // letter can be changed freely
                                plane[px as usize][py as usize] = get_random_letter_byte();
                                cell_fix_made = true;
                                break;
                            } else {
                                panic!("Failed to traverse word found in grid");
                            }
                        }
                        // if false, then an unrecoverable state has been reached
                        if !cell_fix_made {
                            repair_failed = true;
                            break;
                        }
                    }
                    // if true, then the word has multiple occurrences stemming from the same point, which is an unrecoverable state
                    if duplicates_overlap {
                        repair_failed = true;
                        break;
                    }
                }
            }
            if no_duplicates_found {
                return true;
            }
            if repair_failed {
                return false;
            }
        }
        false
    }
}
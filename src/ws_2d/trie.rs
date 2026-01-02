use rand::Rng;
use std::collections::HashMap;
use crate::common::util::{validate_word,get_random_letter_byte,get_cell,Direction,DIRECTIONS,EMPTY};

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
        Grid layout: (y increases from left to right, x increases from top to bottom)
        +---------y
        |
        |
        |
        |
        x
    */
    pub fn get_random(&self, grid : &Vec<Vec<u8>>, x : i32, y : i32, d : (i32,i32), min_len : i32, max_len : i32, words_used : &Vec<String>) -> Option<String> {
        self.helper(grid, x, y, d, min_len, max_len, words_used, 0)
    }

    fn helper(&self, grid : &Vec<Vec<u8>>, x : i32, y : i32, d : (i32,i32), min_len: i32, max_len : i32, words_used : &Vec<String>, iterations : i32) -> Option<String> {
        // exit early if the maximum possible word length is shorter than all words in the trie or if the minimum word length is longer than all words in the trie
        if max_len < self.min_len || min_len > self.max_len {
            return None;
        }
        // exit if the failure count gets too large
        if iterations >= 32 {
            return None;
        }
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        let mut words_found : Vec<String> = vec![];
        let mut curr : Vec<u8> = vec![];
        let mut node : &TrieNode = &self.root;
        let mut pos: i32 = 0;
        // search while (1) position has not exceeded maximum length, and (2) maximum word length in subtrie is at least the minimum required length
        // also checking that trie node is valid
        while pos < max_len && node.max_len >= min_len {
            // check if shortest possible word from the current node will not fit (out of bounds)
            if get_cell(grid, x, y, d, node.min_len-1).is_none() {
                break;
            };
            let cell: u8 = match get_cell(grid, x, y, d, pos) {
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
           return self.helper(grid, x, y, d, min_len, max_len, words_used, iterations+1);
        }
        if n == 1 {
            return Some(words_found[0].clone());
        }
        // use weighted randomization (favoring longer words)
        // weights are 2,3,...,n+1. sum = ((n+2)(n+1)/2)-1
        let total: i32 = ((n+2)*(n+1)/2)-1;
        let r : i32 = rng.random_range(0..total);
        let mut curr: i32 = 0;
        for i in 2..=(n+1) {
            curr += i;
            if curr > r {
                return Some(words_found[(i-2) as usize].clone());
            }
        }
        panic!("Random selection from words found failed");
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
    pub fn verify_and_repair_grid(&mut self, grid : &mut Vec<Vec<u8>>, words : &Vec<String>, word_pos : &HashMap<String,(i32,i32)>, dir_map : &HashMap<(i32,i32),Vec<Direction>>) -> bool {
        // map format: { word : [(x,y,direction)] }, direction: (dx,dy)
        let mut freq : HashMap<&str,Vec<(i32,i32,(i32,i32))>> = HashMap::new();
        for w in words {
            freq.insert(w, vec![]);
        }
        let dim : i32  = grid.len() as i32;
        for x in 0..dim {
            for y in 0..dim {
                for d in DIRECTIONS {
                    // perform out-of-bounds check
                    let px: i32 = x + d.0 * (self.root.min_len-1);
                    let py: i32 = y + d.1 * (self.root.min_len-1);
                    if px < 0 || py < 0 || px >= dim || py >= dim {
                        continue;
                    }
                    let mut node: &mut TrieNode = &mut self.root;
                    let mut curr : Vec<u8> = vec![];
                    let mut pos: i32 = 0;
                    loop {
                        let px : i32 = x + d.0 * pos;
                        let py : i32 = y + d.1 * pos;
                        if px < 0 || py < 0 || px >= dim || py >= dim {
                            break;
                        }
                        let b: u8 = grid[px as usize][py as usize];
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
                                Some(v) => v.push((x,y,d)),
                                None => panic!("Error occurred when updating frequency map"),
                            }
                            break;
                        }
                        pos+=1;
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
                let true_pos: (i32, i32) = match word_pos.get(word) {
                    None => panic!("Word has no position"),
                    Some(&p) => p
                };
                no_duplicates_found = false;
                let mut duplicates_overlap: bool = true;
                for (x,y,d) in occurrences {
                    // Check if current occurrence matches the word's stored position
                    if &true_pos.0 == x && &true_pos.1 == y {
                        continue;
                    }
                    // Stored position not matched, so grid must be verified again
                    // Attempt to make changes where appropriate
                    duplicates_overlap = false;
                    let n: usize = word.len();
                    let mut cell_fix_made: bool = false;
                    for i in 0..n {
                        let px: i32 = x + d.0 * (i as i32);
                        let py: i32 = y + d.1 * (i as i32);
                        // check if letter belongs to a word
                        if dir_map.contains_key(&(px,py)) {
                            continue;
                        }
                        // letter can be changed freely
                        grid[px as usize][py as usize] = get_random_letter_byte();
                        cell_fix_made = true;
                        break;
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
        // Cannot guarantee that the grid is correct, so try again
        self.verify_and_repair_grid(grid, words, word_pos, dir_map)
    }
}
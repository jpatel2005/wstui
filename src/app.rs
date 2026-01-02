use std::{collections::HashSet, time::{Instant, Duration}};
use crate::common::util::{get_pos_3d, DIRECTIONS};
use crate::ws_2d;
use crate::ws_3d::{self, grid::Face};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf
};
use serde::{Serialize, Deserialize};
use chrono::Local;
use directories::ProjectDirs;

pub enum ActivePuzzle {
    TwoD(ws_2d::puzzle::WordSearch),
    ThreeD(ws_3d::puzzle::WordSearch),
}

impl ActivePuzzle {
    pub fn get_words(&self) -> &Vec<String> {
        match self {
            ActivePuzzle::TwoD(ws) => &ws.words,
            ActivePuzzle::ThreeD(ws) => &ws.words,
        }
    }
}

pub enum ActiveTrie {
    TwoD(ws_2d::trie::Trie),
    ThreeD(ws_3d::trie::Trie),
}

// APP STATES

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Menu,
    Game,
    NewPuzzlePopup,
    Win,
    SavePuzzlePopup,
    ExitWarning,
    LoadPuzzlePopup,
    DeleteSavePopup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleType {
    TwoD,
    ThreeD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupFocus {
    TypeSelector,
    SizeSelector,
    Buttons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    NewPuzzle,
    LoadPuzzle,
    // Rules,
    Quit,
}

pub struct SaveFile {
    pub filename: String,
    pub timestamp_str: String,
    pub puzzle_type: String,
    pub puzzle_size: String,
    pub found: usize,
    pub total: usize,
    pub elapsed: u64,
}

#[derive(Serialize, Deserialize)]
struct SaveState {
    puzzle_type: String,
    puzzle_size: String,
    grid_2d: Option<Vec<Vec<u8>>>,    
    grid_3d: Option<[Vec<Vec<u8>>; 3]>, // [top,right,left]
    words: Vec<String>,
    found_words: Vec<String>,
    found_cells: Vec<(usize, usize, usize)>,
    elapsed_seconds: u64,
}

// MAIN APP STRUCT

pub struct App {
    pub exit: bool,
    pub current_screen: CurrentScreen,
    pub selected_item: usize,
    pub menu_items: Vec<MenuItem>,
    // New Puzzle Settings
    pub puzzle_type: PuzzleType,
    pub puzzle_size: PuzzleSize,
    pub popup_focus: PopupFocus,
    pub popup_button_index: usize,
    // Word Search Data
    pub word_search: ActivePuzzle,
    init_done: bool,
    word_trie: ActiveTrie,
    // Puzzle related
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_z: usize,
    pub size: usize,

    pub scroll_offset: usize,
    pub list_height: usize,

    pub found_words: Vec<String>,
    pub show_found_words: bool,

    pub found_cells: HashSet<(usize, usize, usize)>, 
    pub selection_start: Option<(usize, usize, usize)>,

    pub start_time: Option<Instant>,
    pub finish_time: Option<Duration>,

    pub notification: Option<String>,
    pub notification_time: Option<Instant>,

    pub save_files: Vec<SaveFile>,
    pub selected_save_index: usize,
}

impl App {
pub fn new() -> Self {
        let default_puzzle = ws_2d::puzzle::WordSearch { grid: Vec::new(), words: Vec::new() };
        let default_trie = ws_2d::trie::Trie::new();
        Self {
            exit: false,
            current_screen: CurrentScreen::Menu,
            selected_item: 0,
            menu_items: vec![
                MenuItem::NewPuzzle,
                MenuItem::LoadPuzzle,
                // MenuItem::Rules,
                MenuItem::Quit,
            ],
            puzzle_type: PuzzleType::TwoD, 
            puzzle_size: PuzzleSize::Medium,
            popup_focus: PopupFocus::TypeSelector,
            popup_button_index: 0,

            word_search: ActivePuzzle::TwoD(default_puzzle),
            init_done: false,
            word_trie: ActiveTrie::TwoD(default_trie),

            cursor_x: 0,
            cursor_y: 0,
            cursor_z: 0,
            size: 0,
            
            scroll_offset: 0,
            list_height: 0,

            show_found_words: true,
            found_words: Vec::new(),

            found_cells: HashSet::new(),
            selection_start: None,

            start_time: None,
            finish_time: None,

            notification: None,
            notification_time: None,

            save_files: Vec::new(),
            selected_save_index: 0,
        }
    }

    pub fn start_game(&mut self) {
        self.current_screen = CurrentScreen::Game;
        self.generate_grid();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.cursor_z = 0;
        self.selection_start = None;
        self.found_words.clear();
        self.found_cells.clear();
        self.scroll_offset = 0;
        self.show_found_words = true;
        self.start_time = Some(Instant::now());
        self.finish_time = None;
    }

    fn generate_grid(&mut self) {
        let size = match self.puzzle_size {
            PuzzleSize::Small => match self.puzzle_type { PuzzleType::TwoD => 20, PuzzleType::ThreeD => 8 },
            PuzzleSize::Medium => match self.puzzle_type { PuzzleType::TwoD => 30, PuzzleType::ThreeD => 10 },
            PuzzleSize::Large => match self.puzzle_type { PuzzleType::TwoD => 40, PuzzleType::ThreeD => 12 },
        };
        match self.puzzle_type {
            PuzzleType::TwoD => {
                // Initialize if not done OR if the current trie is actually a 3D trie
                if !self.init_done || matches!(self.word_trie, ActiveTrie::ThreeD(_)) {
                    self.word_trie = ActiveTrie::TwoD(ws_2d::puzzle::init());
                    self.init_done = true;
                }
            }
            PuzzleType::ThreeD => {
                // Initialize if not done OR if the current trie is actually a 2D trie
                if !self.init_done || matches!(self.word_trie, ActiveTrie::TwoD(_)) {
                    self.word_trie = ActiveTrie::ThreeD(ws_3d::puzzle::init());
                    self.init_done = true;
                }
            }
        }
        // generate grid
        match self.puzzle_type {
            PuzzleType::TwoD => {
                if let ActiveTrie::TwoD(trie) = &self.word_trie {
                    let mut ws = ws_2d::puzzle::generate(size, trie);
                    ws.words.sort();
                    self.word_search = ActivePuzzle::TwoD(ws);
                }
            }
            PuzzleType::ThreeD => {
                if let ActiveTrie::ThreeD(trie) = &self.word_trie {
                    let mut ws = ws_3d::puzzle::generate(size, trie);
                    ws.words.sort();
                    self.word_search = ActivePuzzle::ThreeD(ws);
                }
            }
        }
        self.size = size;
    }

    // Cursor Movement w/ wrapping when appropriate

    pub fn move_cursor_left(&mut self) {
        match self.puzzle_type {
            PuzzleType::TwoD => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                } else {
                    self.cursor_y = self.size-1;
                }
            }
            PuzzleType::ThreeD => self.move_cursor_3d(0, -1),
        }
    }

    pub fn move_cursor_right(&mut self) {
        match self.puzzle_type {
            PuzzleType::TwoD => {
                if let ActivePuzzle::TwoD(ws) = &self.word_search {
                    if self.cursor_y < ws.grid.len()-1 {
                        self.cursor_y += 1;
                    } else {
                        self.cursor_y = 0;
                    }
                }
            }
            PuzzleType::ThreeD => self.move_cursor_3d(0, 1),
        }
    }

    pub fn move_cursor_up(&mut self) {
        match self.puzzle_type {
            PuzzleType::TwoD => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                } else {
                    self.cursor_x = self.size-1;
                }
            }
            PuzzleType::ThreeD => self.move_cursor_3d(-1, 0),
        }
    }

    pub fn move_cursor_down(&mut self) {
        match self.puzzle_type {
            PuzzleType::TwoD => {
                if let ActivePuzzle::TwoD(ws) = &self.word_search {
                    if self.cursor_x < ws.grid[0].len()-1 {
                        self.cursor_x += 1;
                    } else {
                        self.cursor_x = 0;
                    }
                }
            }
            PuzzleType::ThreeD => self.move_cursor_3d(1, 0),
        }
    }

    // 3D navigation logic
    
    fn move_cursor_3d(&mut self, dx: i32, dy: i32) {
        let size = match &self.word_search {
            ActivePuzzle::ThreeD(ws) => ws.grid.get_size() as i32,
            _ => return, // should not happen
        };
        let face = self.cursor_z; // 0=Top, 1=Right, 2=Left
        let mut r = self.cursor_x as i32;
        let mut c = self.cursor_y as i32;
        // process face by cases (0-2)
        match face {
            0 => { // TOP FACE
                r += dx;
                c += dy;
                if r >= size { 
                    self.cursor_z = 2;
                    self.cursor_x = 0;
                } else if c >= size {
                    self.cursor_z = 1;
                    self.cursor_y = (size as usize)-1-self.cursor_x;
                    self.cursor_x = 0;
                } else {
                    if r < 0 { r = size-1; }
                    if c < 0 { c = size-1; }
                    self.cursor_x = r as usize;
                    self.cursor_y = c as usize;
                }
            }
            1 => { // RIGHT FACE
                r += dx;
                c += dy;
                if r < 0 {
                    self.cursor_z = 0;
                    self.cursor_x = (size as usize)-1-self.cursor_y; 
                    self.cursor_y = (size as usize)-1;
                } else if c < 0 {
                    self.cursor_z = 2;
                    self.cursor_y = (size as usize)-1;
                } else {
                    if r >= size { r = 0 }
                    if c >= size { c = 0 }
                    self.cursor_x = r as usize;
                    self.cursor_y = c as usize;
                }
            }
            2 => { // LEFT FACE
                r += dx;
                c += dy;
                if r < 0 {
                    self.cursor_z = 0;
                    self.cursor_x = (size as usize)-1;
                    self.cursor_y = c as usize;
                } else if c >= size {
                    self.cursor_z = 1;
                    self.cursor_y = 0;
                } else {
                    if r >= size { r = 0 }
                    if c < 0 { c = size-1 }
                    self.cursor_x = r as usize;
                    self.cursor_y = c as usize;
                }
            }
            _ => {}
        }
    }

    // LIST SCROLLING

    pub fn scroll_list_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_list_down(&mut self) {
        if self.scroll_offset + self.list_height < self.word_search.get_words().len() {
            self.scroll_offset += 1;
        }
    }

    // SELECTION LOGIC

    // Finds the line(s) of cells (if there are any) from the selection start to the current cell
    pub fn get_current_selection_lines(&self) -> Vec<Vec<(usize, usize, usize)>> {
        if let Some(start) = self.selection_start {
            let end = (self.cursor_x, self.cursor_y, self.cursor_z);
            if start == end {
                return vec![vec![start]];
            }
            match self.puzzle_type {
                PuzzleType::TwoD => self.get_line_2d(start, end),
                PuzzleType::ThreeD => self.get_lines_3d(start, end),
            }
        } else { Vec::new() }
    }

    fn get_line_2d(&self, start: (usize, usize, usize), end: (usize, usize, usize)) -> Vec<Vec<(usize, usize, usize)>> {
        let (x1, y1, _) = start;
        let (x2, y2, _) = end;
        let dx = (x2 as isize) - (x1 as isize);
        let dy = (y2 as isize) - (y1 as isize);
        // check for straight or diagonal lines
        if dx == 0 || dy == 0 || dx.abs() == dy.abs() {
            let steps = dx.abs().max(dy.abs());
            let step_x = dx.signum();
            let step_y = dy.signum();
            let mut line = Vec::new();
            for i in 0..=steps {
                let nx = (x1 as isize) + i*step_x;
                let ny = (y1 as isize) + i*step_y;
                line.push((nx as usize, ny as usize, 0));
            }
            vec![line]
        } else {
            Vec::new()
        }
    }

    // TODO Consider optimizing this (only 3 directions are necessary to check)
    fn get_lines_3d(&self, start: (usize, usize, usize), end: (usize, usize, usize)) -> Vec<Vec<(usize, usize, usize)>> {
        let grid = match &self.word_search {
            ActivePuzzle::ThreeD(ws) => &ws.grid,
            _ => return Vec::new(),
        };
        let (start_row, start_col, start_z) = start;
        let (end_row, end_col, end_z) = end;
        let start_logic_x = start_row as i32;
        let start_logic_y = start_col as i32;
        let start_face = match start_z { 
            0 => Face::Top, 
            1 => Face::Right, 
            _ => Face::Left 
        };
        let max_steps = (grid.get_size() as i32) * 3; 
        let mut valid_paths = Vec::new();
        for &(dx, dy) in &DIRECTIONS {
            let mut path = Vec::new();
            let mut matched = false;
            for step in 0..=max_steps {
                let pos_res = get_pos_3d(
                    grid, 
                    start_face, 
                    start_logic_x, 
                    start_logic_y, 
                    (dx, dy), 
                    step
                );
                if let Some((curr_face, lx, ly, _)) = pos_res {
                    let new_row = lx as usize;
                    let new_col = ly as usize;
                    let new_z = match curr_face { 
                        Face::Top => 0, 
                        Face::Right => 1, 
                        Face::Left => 2 
                    };
                    path.push((new_row, new_col, new_z));
                    if new_col == end_col && new_row == end_row && new_z == end_z {
                        matched = true;
                        break;
                    }
                } else {
                    // invalid path
                    break;
                }
            }
            if matched {
                valid_paths.push(path);
                if valid_paths.len() == 2 {
                    break;
                }
            }
        }
        valid_paths
    }

    // Old implementation (failed to consider multiple lines)    
    fn _get_line_3d(&self, start: (usize, usize, usize), end: (usize, usize, usize)) -> Option<Vec<(usize, usize, usize)>> {
        let grid = match &self.word_search {
            ActivePuzzle::ThreeD(ws) => &ws.grid,
            _ => return None,
        };
        let (start_row, start_col, start_z) = start;
        let (end_row, end_col, end_z) = end;
        let start_logic_x = start_row as i32;
        let start_logic_y = start_col as i32;
        let start_face = match start_z { 
            0 => Face::Top, 
            1 => Face::Right, 
            _ => Face::Left 
        };
        let max_steps = (grid.get_size() as i32) * 3; 
        for &(dx, dy) in &DIRECTIONS {
            let mut path = Vec::new();
            let mut matched = false;
            for step in 0..=max_steps {
                let pos_res = get_pos_3d(
                    grid, 
                    start_face, 
                    start_logic_x, 
                    start_logic_y, 
                    (dx, dy), 
                    step
                );
                if let Some((curr_face, lx, ly, _)) = pos_res {
                    let new_row = lx as usize;
                    let new_col = ly as usize;
                    let new_z = match curr_face { 
                        Face::Top => 0, 
                        Face::Right => 1, 
                        Face::Left => 2 
                    };
                    path.push((new_row, new_col, new_z));
                    if new_col == end_col && new_row == end_row && new_z == end_z {
                        matched = true;
                        break;
                    }
                } else {
                    // invalid path
                    break;
                }
            }
            if matched {
                return Some(path);
            }
        }

        None
    }

    pub fn toggle_selection(&mut self) {
        if let Some(_) = self.selection_start {
            // Ending selection
            // Need to check if the selection is a line and is a word in our word list (also check reverese direction)
            let all_paths = self.get_current_selection_lines();
            if !all_paths.is_empty() {
                let mut found_match = false;
                // 0-1 paths for 2d case, 0-2 paths for 3d case
                for path in &all_paths {
                    let mut curr_selection = String::new();
                    match self.word_search {
                        ActivePuzzle::TwoD(ref ws) => {
                            for &(x, y, _) in path {
                                curr_selection.push(ws.grid[x][y] as char);
                            }
                        }
                        ActivePuzzle::ThreeD(ref ws) => {
                            for &(x, y, z) in path {
                                let face = match z { 0 => Face::Top, 1 => Face::Right, _ => Face::Left };
                                curr_selection.push(ws.grid.get_face(face)[x][y] as char);
                            }
                        }
                    }
                    let rev_selection : String = curr_selection.chars().rev().collect();              
                    let found_word = self.word_search.get_words().iter().find(|&w| {
                        w.eq_ignore_ascii_case(&curr_selection) || w.eq_ignore_ascii_case(&rev_selection)
                    });
                    if let Some(word) = found_word {
                        let w_str = word.clone();
                        if !self.found_words.contains(&w_str) {
                            self.found_words.push(w_str.clone());
                            for &cell in path {
                                self.found_cells.insert(cell);
                            }
                            self.notification = Some(format!("Found: {}", w_str));
                            self.notification_time = Some(Instant::now());
                            found_match = true;
                        }
                    }
                }
                // check for win
                if found_match {
                    let total_words = self.word_search.get_words().len();
                    if self.found_words.len() == total_words {
                        self.current_screen = CurrentScreen::Win;
                        if let Some(start) = self.start_time {
                            self.finish_time = Some(start.elapsed());
                        }
                    }
                }
            }
            // reset selection
            self.selection_start = None;
        } else {
            // start selection
            self.selection_start = Some((self.cursor_x, self.cursor_y, self.cursor_z));
        }
    }

    pub fn toggle_found_visibility(&mut self) {
        self.show_found_words = !self.show_found_words;
    }

    pub fn open_save_popup(&mut self) {
        self.current_screen = CurrentScreen::SavePuzzlePopup;
    }

    pub fn close_save_popup(&mut self) {
        self.current_screen = CurrentScreen::Game;
    }
    
    fn get_save_directory() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "YourName", "wstui") {
            let save_dir = proj_dirs.data_dir().join("saves");            
            let _ = std::fs::create_dir_all(&save_dir);            
            return save_dir;
        }
        PathBuf::from(".")
    }

    pub fn save_puzzle(&mut self) {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let mut filepath = Self::get_save_directory();
        filepath.push(format!("puzzle-{}.json", timestamp));
        let elapsed = if let Some(start) = self.start_time { start.elapsed().as_secs() } else { 0 };
        let size_str = match self.puzzle_size {
            PuzzleSize::Small => "Small",
            PuzzleSize::Medium => "Medium",
            PuzzleSize::Large => "Large",
        }.to_string();
        let mut save_data = SaveState {
            puzzle_type: String::new(),
            puzzle_size: size_str,
            grid_2d: None,
            grid_3d: None,
            words: self.word_search.get_words().clone(),
            found_words: self.found_words.clone(),
            found_cells: self.found_cells.iter().cloned().collect(),
            elapsed_seconds: elapsed,
        };
        // extract grid data
        match &self.word_search {
            ActivePuzzle::TwoD(ws) => {
                save_data.puzzle_type = "TwoD".to_string();
                save_data.grid_2d = Some(ws.grid.to_vec());
            }
            ActivePuzzle::ThreeD(ws) => {
                save_data.puzzle_type = "ThreeD".to_string();
                let top = ws.grid.get_face(Face::Top).to_vec();
                let right = ws.grid.get_face(Face::Right).to_vec();
                let left = ws.grid.get_face(Face::Left).to_vec();                
                save_data.grid_3d = Some([top, right, left]);
            }
        }
        // serialize and write
        if let Ok(json) = serde_json::to_string(&save_data) {
            if let Ok(mut file) = File::create(&filepath) {
                if file.write_all(json.as_bytes()).is_ok() {
                    self.notification = Some(format!("Puzzle saved: puzzle-{}", timestamp));
                    self.notification_time = Some(Instant::now());
                }
            }
        }
        self.current_screen = CurrentScreen::Game;
    }

    pub fn open_load_popup(&mut self) {
        self.scan_save_files();
        if !self.save_files.is_empty() {
            self.selected_save_index = 0;
        }
        self.current_screen = CurrentScreen::LoadPuzzlePopup;
    }

    pub fn close_load_popup(&mut self) {
        self.current_screen = CurrentScreen::Menu;
    }

    pub fn nav_load_up(&mut self) {
        if !self.save_files.is_empty() && self.selected_save_index > 0 {
            self.selected_save_index -= 1;
        }
    }

    pub fn nav_load_down(&mut self) {
        if !self.save_files.is_empty() && self.selected_save_index < self.save_files.len()-1 {
            self.selected_save_index += 1;
        }
    }

    pub fn load_selected_puzzle(&mut self) {
        if self.save_files.is_empty() { return; }        
        let filename = &self.save_files[self.selected_save_index].filename;        
        let mut path = Self::get_save_directory();
        path.push(filename);
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let save_data: SaveState = match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(_) => return,
        };
        // restore found words/cells
        self.found_words = save_data.found_words.clone();
        self.found_cells = save_data.found_cells.iter().cloned().collect();
        // restore timer
        self.start_time = Some(Instant::now() - Duration::from_secs(save_data.elapsed_seconds));
        self.finish_time = None;
        // resize size
        self.puzzle_size = match save_data.puzzle_size.as_str() {
            "Small" => PuzzleSize::Small,
            "Large" => PuzzleSize::Large,
            _ => PuzzleSize::Medium,
        };
        // restore puzzle stuff (Grid + Trie)
        match save_data.puzzle_type.as_str() {
            "TwoD" => {
                self.puzzle_type = PuzzleType::TwoD;
                if let Some(grid_u8) = save_data.grid_2d {
                    let mut trie = ws_2d::trie::Trie::new();
                    for word in &save_data.words {
                        trie.insert(word);
                    }
                    self.word_trie = ActiveTrie::TwoD(trie);
                    self.word_search = ActivePuzzle::TwoD(ws_2d::puzzle::WordSearch {
                        grid: grid_u8,
                        words: save_data.words,
                    });
                }
            },
            "ThreeD" => {
                self.puzzle_type = PuzzleType::ThreeD;
                if let Some(faces) = save_data.grid_3d {
                    let top = &faces[0];
                    let right = &faces[1];
                    let left = &faces[2];
                    let mut trie = ws_3d::trie::Trie::new();
                    for word in &save_data.words {
                        trie.insert(word);
                    }
                    self.word_trie = ActiveTrie::ThreeD(trie);
                    let mut grid_3d = ws_3d::grid::Grid3D::new(top.len());
                    grid_3d.set_faces(left.to_vec(), right.to_vec(), top.to_vec());
                    self.word_search = ActivePuzzle::ThreeD(ws_3d::puzzle::WordSearch {
                        grid: grid_3d,
                        words: save_data.words,
                    });
                }
            },
            _ => return,
        }
        // reset cursor and switch screen
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.cursor_z = 0;
        self.selection_start = None;
        self.current_screen = CurrentScreen::Game;
    }

    fn scan_save_files(&mut self) {
        self.save_files.clear();
        let save_dir = Self::get_save_directory();
        if let Ok(entries) = fs::read_dir(save_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with("puzzle-") && filename.ends_with(".json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                let p_type = json["puzzle_type"].as_str().unwrap_or("?").to_string();
                                let p_size = json["puzzle_size"].as_str().unwrap_or("?").to_string();
                                let found = json["found_words"].as_array().map(|v| v.len()).unwrap_or(0);
                                let total = json["words"].as_array().map(|v| v.len()).unwrap_or(1);
                                let elapsed = json["elapsed_seconds"].as_u64().unwrap_or(0);
                                let date_part = &filename[7..filename.len()-5]; // YYYYMMDD-HHMMSS
                                // check display date formatting
                                let display_date = if date_part.len() == 15 {
                                    // YYYY-MM-DD HH:MM:SS
                                    format!("{}-{}-{} {}:{}:{}", 
                                        &date_part[0..4], &date_part[4..6], &date_part[6..8],
                                        &date_part[9..11], &date_part[11..13], &date_part[13..15])
                                } else {
                                    date_part.to_string()
                                };
                                self.save_files.push(SaveFile {
                                    filename: filename.to_string(),
                                    timestamp_str: display_date,
                                    puzzle_type: p_type,
                                    puzzle_size: p_size,
                                    found,
                                    total,
                                    elapsed,
                                });
                            }
                        }
                    }
                }
            }
        }
        // newest files first
        self.save_files.sort_by(|a, b| b.filename.cmp(&a.filename));
    }

    pub fn open_delete_popup(&mut self) {
        if !self.save_files.is_empty() {
            self.current_screen = CurrentScreen::DeleteSavePopup;
        }
    }

    pub fn close_delete_popup(&mut self) {
        self.current_screen = CurrentScreen::LoadPuzzlePopup;
    }

    pub fn delete_selected_save(&mut self) {
        if self.save_files.is_empty() { return; }
        let filename = &self.save_files[self.selected_save_index].filename;
        let mut path = Self::get_save_directory();
        path.push(filename);
        let _ = std::fs::remove_file(path);
        self.scan_save_files();
        if self.selected_save_index >= self.save_files.len() {
            if !self.save_files.is_empty() {
                self.selected_save_index = self.save_files.len() - 1;
            } else {
                self.selected_save_index = 0;
            }
        }
        self.current_screen = CurrentScreen::LoadPuzzlePopup;
    }

    pub fn trigger_exit_warning(&mut self) {
        self.current_screen = CurrentScreen::ExitWarning;
    }

    pub fn cancel_exit(&mut self) {
        self.current_screen = CurrentScreen::Game;
    }

    pub fn confirm_exit(&mut self) {
        self.current_screen = CurrentScreen::Menu;
    }

    pub fn nav_up(&mut self) {
        if self.selected_item > 0 {
            self.selected_item -= 1;
        } else {
            self.selected_item = self.menu_items.len()-1;
        }
    }

    pub fn nav_down(&mut self) {
        if self.selected_item < self.menu_items.len()-1 {
            self.selected_item += 1;
        } else {
            self.selected_item = 0;
        }
    }

    pub fn quit(&mut self) {
        self.exit = true;
    }
}
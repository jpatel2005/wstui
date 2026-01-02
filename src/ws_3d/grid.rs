use rand::Rng;
use crate::common::util::EMPTY;

#[derive(Copy,Clone,Eq, Hash, PartialEq)]
pub enum Face {
    Left,
    Right,
    Top
}

impl Face {
    pub fn get_random() -> Face {
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        let idx: usize = rng.random_range(0..3);
        if idx == 0 { Face::Left } else if idx == 1 { Face::Right } else { Face::Top }
    }
    pub fn iterator() -> std::slice::Iter<'static, Face> {
        static FACES: [Face; 3] = [Face::Left, Face::Right, Face::Top];
        FACES.iter()
    }
}

pub struct Grid3D {
    left: Vec<Vec<u8>>,
    right: Vec<Vec<u8>>,
    top: Vec<Vec<u8>>,
    size : usize,
}

impl Grid3D {
    pub fn new(n : usize) -> Self {
        Grid3D { 
            left: vec![vec![EMPTY; n]; n],
            right: vec![vec![EMPTY; n]; n],
            top: vec![vec![EMPTY; n]; n],
            size: n,
        }
    }
    pub fn set_faces(&mut self, left : Vec<Vec<u8>>, right : Vec<Vec<u8>>, top : Vec<Vec<u8>>) {
        self.left = left;
        self.right = right;
        self.top = top;
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn get_face(&self, face : Face) -> &Vec<Vec<u8>> {
        match face {
            Face::Left => &self.left,
            Face::Right => &self.right,
            Face::Top => &self.top,
        }
    }
    pub fn get_face_mut(&mut self, face : Face) -> &mut Vec<Vec<u8>> {
        match face {
            Face::Left => &mut self.left,
            Face::Right => &mut self.right,
            Face::Top => &mut self.top,
        }
    }
    pub fn is_empty(&self) -> bool {
        for &face in Face::iterator() {
            if !self.get_face(face).is_empty() {
                return false;
            }
        }
        true
    }
}
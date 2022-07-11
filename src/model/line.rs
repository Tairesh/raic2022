use crate::model::Vec2;

pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        self.start.distance(&self.end)
    }

    pub fn as_vec(&self) -> Vec2 {
        self.end - self.start
    }

    pub fn set_length(&mut self, length: f64) {
        let vec = self.as_vec().normalize() * length;
        self.end = self.start + vec;
    }
}

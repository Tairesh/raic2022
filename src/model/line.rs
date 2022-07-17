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
        self.start.distance_to(&self.end)
    }

    pub fn as_vec(&self) -> Vec2 {
        self.end - self.start
    }

    pub fn set_length(&mut self, length: f64) {
        let vec = self.as_vec().normalize() * length;
        self.end = self.start + vec;
    }

    pub fn distance_to_point(&self, point: &Vec2) -> f64 {
        let vec_len = self.length();
        if vec_len == 0.0 {
            return point.distance_to(&self.start);
        }
        let vec_norm = self.as_vec().normalize();
        let vec_norm_dot_point = vec_norm.dot(&(*point - self.start));
        if vec_norm_dot_point < 0.0 {
            return point.distance_to(&self.start);
        }
        if vec_norm_dot_point > vec_len {
            return point.distance_to(&self.end);
        }
        let point_on_line = self.start + vec_norm * vec_norm_dot_point;
        point.distance_to(&point_on_line)
    }
}

use crate::model::{Line, Vec2};

pub struct Circle {
    pub center: Vec2,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Vec2, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn intersects_with(&self, other: &Self) -> bool {
        self.center.distance(&other.center) <= self.radius + other.radius
    }

    pub fn contains(&self, point: &Vec2) -> bool {
        self.center.distance(point) <= self.radius
    }

    pub fn intercept_with_line(&self, line: &Line) -> bool {
        let a = line.end - line.start;
        let b = self.center - line.start;
        let c = a.dot(&b) / a.dot(&a);
        if c < 0.0 || c > 1.0 {
            return false;
        }
        let p = line.start + a * c;
        self.contains(&p)
    }
}

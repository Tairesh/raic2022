use crate::model::Vec2;

/// 2 dimensional vector.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Vec2i {
    /// `x` coordinate of the vector
    pub x: i32,
    /// `y` coordinate of the vector
    pub y: i32,
}

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn square_distance(&self, other: &Self) -> i32 {
        (self.x - other.x).pow(2) + (self.y - other.y).pow(2)
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl From<Vec2i> for Vec2 {
    fn from(point: Vec2i) -> Self {
        Self::new(point.x as f64, point.y as f64)
    }
}

impl From<Vec2> for Vec2i {
    fn from(point: Vec2) -> Self {
        Self::new(point.x.round() as i32, point.y.round() as i32)
    }
}

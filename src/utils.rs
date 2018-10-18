use consts::*;
use ggez::graphics::{Color, Point2};

//////////////////////////////////////////////////

macro_rules! sqr {
    ($x:expr) => {
        $x * $x
    };
}

pub fn green() -> Color {
    Color::new(0., 1., 0., 1.)
}
pub fn red() -> Color {
    Color::new(1., 0., 0., 1.)
}
pub fn blue() -> Color {
    Color::new(0., 0., 1., 1.)
}

// fn shoot_get_1d_velocity(distance: f32, theta: f32) -> f32 {
//     let x = distance * GRAVITY / (theta * 2.0).sin();
//     x.sqrt()
// }

pub trait PointArithmetic: Sized {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn rotation(self) -> f32;
    fn dist(self, other: Self) -> f32;
    fn offset(self) -> f32;
}

impl PointArithmetic for Point2 {
    fn add(self, other: Self) -> Self {
        Point2::new(self[0] + other[0], self[1] + other[1])
    }
    fn sub(self, other: Self) -> Self {
        Point2::new(self[0] - other[0], self[1] - other[1])
    }
    fn rotation(self) -> f32 {
        self[1].atan2(self[0])
    }
    fn dist(self, other: Self) -> f32 {
        self.sub(other).offset()
    }
    fn offset(self) -> f32 {
        (sqr!(self[0]) + sqr!(self[1])).sqrt()
    }
}

use consts::*;
use ggez::{
    self,
    graphics::{Color, Point2},
};
use std;

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

fn shoot_get_1d_velocity(distance: f32, theta: f32) -> f32 {
    let x = distance * GRAVITY / (theta * 2.0).sin();
    x.sqrt()
}

pub trait PointArithmetic {
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

// pub mod pt {
//     use super::*;
//     // pub type Point2 = ::nalgebra::Point2<f32>;

//     // pub fn scale(p: Point2, scalar: f32) -> Point2 {
//     //     Point2::new(p[0] * scalar, p[1] * scalar)
//     // }

//     pub fn rotation(pt: Point2) -> f32 {
//         pt[1].atan2(pt[0])
//     }

//     pub fn dist(a: Point2, b: Point2) -> f32 {
//         offset(sub(a, b))
//     }

//     pub fn offset(p: Point2) -> f32 {
//         (sqr!(p[0]) + sqr!(p[1])).sqrt()
//     }

//     pub fn sub(a: Point2, b: Point2) -> Point2 {
//         Point2::new(a[0] - b[0], a[1] - b[1])
//     }

//     pub fn add(a: Point2, b: Point2) -> Point2 {
//         Point2::new(a[0] + b[0], a[1] + b[1])
//     }
// }

// struct Point2Wrapper(Point2);
// impl std::convert::Into<Point2> for Point2Wrapper {
//     fn into(self) -> Point2 {
//         self.0
//     }
// }

// impl std::convert::From<Point2> for Point2Wrapper {
//     fn from(p: Point2) -> Self {
//         Point2Wrapper(p)
//     }
// }

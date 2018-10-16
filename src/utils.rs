use ggez::{
	self,
	graphics::{
		Point2,
		Color,
	},
};
use consts::*;

//////////////////////////////////////////////////

macro_rules! sqr {
    ($x:expr) => ($x*$x)
}


pub fn green() -> Color {Color::new(0., 1., 0., 1.)}
pub fn red() -> Color {Color::new(1., 0., 0., 1.)}
pub fn blue() -> Color {Color::new(0., 0., 1., 1.)}

fn shoot_get_1d_velocity(distance: f32, theta: f32) -> f32 {
    let x = distance * GRAVITY / (theta * 2.0).sin();
    x.sqrt()
}

pub mod pt {
	use super::*;

	pub fn scale(p: Point2, scalar: f32) -> Point2 {
	    Point2::new(p[0]*scalar, p[1]*scalar)
	}

	pub fn rotation(pt: Point2) -> f32 {
	    pt[1].atan2(pt[0])
	}


	pub fn dist(a: Point2, b: Point2) -> f32 {
	    offset(sub(a, b))
	}

	pub fn offset(p: Point2) -> f32 {
	    (
	        sqr!(p[0]) + sqr!(p[1])
	    ).sqrt()
	}

	pub fn sub(a: Point2, b: Point2) -> Point2 {
	    Point2::new(a[0]-b[0], a[1]-b[1])
	}

	pub fn add(a: Point2, b: Point2) -> Point2 {
	    Point2::new(a[0]+b[0], a[1]+b[1])
	}
}
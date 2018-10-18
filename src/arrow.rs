use ggez::graphics::Point2;
use std;

use consts::*;
use std::f32::consts::PI;
use utils::PointArithmetic;

//////////////////////////////////////////////////

pub struct LiveArrow {
    pub position: Point2,
    pub angle: f32,
    pub momentum: Point2,
    pub height: f32,
    pub climb_momentum: f32,
}
impl LiveArrow {
    pub fn new(position: Point2, momentum: Point2, climb_momentum: f32) -> Self {
        Self {
            position,
            momentum,
            climb_momentum,
            angle: momentum.rotation(),
            height: 10.0,
        }
    }

    fn normalized_climb(&self) -> f32 {
        self.climb_momentum / self.total_momentum()
    }

    fn total_momentum(&self) -> f32 {
        (sqr!(self.momentum[0]) + sqr!(self.momentum[0]) + sqr!(self.climb_momentum)).sqrt()
    }

    pub fn shadow_draw_length(&self) -> f32 {
        let x = self._vert_draw_ratio().abs();
        let y = ((x * PI).cos() + 1.) / 2.;
        let val = 0.7 * y + 0.2;
        val * 0.3 * self.momentum[0].abs() / self.momentum.offset() + 0.7 * val
    }

    // returns 1.0 when arrow is shot straight up, -1.0 when shot straight down
    fn _vert_draw_ratio(&self) -> f32 {
        let x = self.angle.sin();
        let hyp = self.climb_momentum.hypot(self.momentum.offset());
        let climb_ratio = self.climb_momentum / hyp;
        let x_influence = 0.5 + 0.5 * x.abs();
        x_influence * climb_ratio
    }

    pub fn image_draw_length(&self) -> f32 {
        //TODO REDO entirely
        // let rat = self._vert_draw_ratio();
        let max_len_at_norm_climb = { self.angle.cos() * 0.6 };
        1. - (max_len_at_norm_climb - self.normalized_climb()).abs()
    }

    pub fn image_angle(&self) -> f32 {
        let pi = std::f32::consts::PI;
        let mut ratio = self._vert_draw_ratio();
        let val = if ratio > 0. {
            // skew UPWARD

            //ensure shortest distance around the clock is in phase
            let n = if self.angle > pi / 2. {
                //positive
                self.angle - pi * 2.
            } else {
                //negative
                self.angle
            };
            // print!("angle:{}\tup:{}\t+ normal:{}\tn:{}", self.angle, ratio, 1.-ratio, n);
            (-pi / 2./*UP*/) * ratio + n * (1. - ratio)
        } else {
            ratio *= -1.;
            // skew DOWNWARD

            //ensure shortest distance around the clock is in phase
            let n = if self.angle < -pi / 2. {
                //positive
                self.angle + pi * 2.
            // self.angle
            } else {
                //negative
                self.angle
            };
            // assert!(n >= 0.);
            // print!("angle:{}\tdown:{}\t+ normal:{}\tn:{}", self.angle, ratio, 1.-ratio, n);

            (pi / 2./*DOWN*/) * ratio + n * (1. - ratio)
        };
        val
    }
}

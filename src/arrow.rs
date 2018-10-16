
use ggez::graphics::Point2;
use std;

use ::consts::*;
use ::utils::{
    self,
    pt,
};

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
            position, momentum, climb_momentum,
            angle: pt::rotation(momentum),
            height: 10.0,
        }
    }

    pub fn shadow_draw_length(&self) -> f32 {
        let pi = std::f32::consts::PI;
        let x = self._vert_draw_ratio().abs();
        let y = ((x*pi).cos()+1.)/2.;
        let val = (0.7*y + 0.2);
        val*0.3*self.momentum[0].abs()/pt::offset(self.momentum) + 0.7*val
    }

    // returns 1.0 when arrow is shot straight up, -1.0 when shot straight down
    fn _vert_draw_ratio(&self) -> f32 {
        let x = self.angle.sin();
        let hyp = self.climb_momentum.hypot(pt::offset(self.momentum));
        let climb_ratio = self.climb_momentum / hyp;
        let x_influence = 0.5 + 0.5*x.abs();
        x_influence*climb_ratio
    }

    pub fn image_draw_length(&self) -> f32 {
        let rat = self._vert_draw_ratio();
        let angle_effect = self.angle.sin();
        let pitch_effet = if angle_effect < 0. {
            //toward camera
            (0.5-rat).abs()
        } else  {
            //away from camera
            (-0.5-rat).abs()
        };
        let effect = angle_effect*pitch_effet;
        let val = effect*1.0 + (1.-effect)*0.7;
        // println!("angle:{}\tangeff:{}\tpitcheff:{}\teffect:{}\tval:{}", self.angle, angle_effect, pitch_effet, effect, val);
        val
    }

    pub fn image_angle(&self) -> f32 {
        let pi = std::f32::consts::PI;
        let mut ratio = self._vert_draw_ratio();
        let val = if ratio > 0. {
            // skew UPWARD

            //ensure shortest distance around the clock is in phase
            let n = if self.angle > pi/2. {
                //positive
                self.angle - pi*2.
            } else {
                //negative
                self.angle
            };
            // print!("angle:{}\tup:{}\t+ normal:{}\tn:{}", self.angle, ratio, 1.-ratio, n);
            (-pi/2./*UP*/)*ratio + n*(1.-ratio)

        } else {
            ratio *= -1.;
            // skew DOWNWARD

            //ensure shortest distance around the clock is in phase
            let n = if self.angle < -pi/2. {
                //positive
                self.angle + pi*2.
                // self.angle
            } else {
                //negative
                self.angle
            };
            // assert!(n >= 0.);
            // print!("angle:{}\tdown:{}\t+ normal:{}\tn:{}", self.angle, ratio, 1.-ratio, n);

            (pi/2./*DOWN*/)*ratio + n*(1.-ratio)
        };
        // println!("\tval:{}", val);
        val
        // selsf.angle
    }
    // fn shadow_angle(&self) -> f32 {
    //     self.angle
    // }
}
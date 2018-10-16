extern crate fnv;
extern crate ggez;
extern crate rand;

mod consts;
use consts::*;

use std::f32::consts::PI;

mod arrow;
mod utils;
use arrow::LiveArrow;
use utils::PointArithmetic;

use ggez::{
    conf,
    event::{self, Keycode, Mod, MouseButton, MouseState},
    graphics::{
        self, spritebatch::SpriteBatch, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Point2,
    },
    timer, Context, GameResult,
};
use std::{env, path};

enum DrawState {
    NotHolding,             // not drawn back
    Nocking(Point2),        // origin
    Drawing(Point2, usize), // origin, turnaround_index
}

struct GameState {
    character_at: Point2,
    live_arrows: Vec<LiveArrow>,
    temp_usize: Vec<usize>,
    arrow_graphic: graphics::Image,
    mouse_pts: Vec<Point2>,
    draw_state: DrawState,
    spot_mesh: Mesh,
    dead_arrows: SpriteBatch,
    dead_arrow_shadows: SpriteBatch,
}

fn default_param() -> DrawParam {
    graphics::DrawParam {
        ..Default::default()
    }
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            live_arrows: vec![],
            temp_usize: vec![],
            character_at: Point2::new(300., 200.),
            mouse_pts: vec![],
            arrow_graphic: graphics::Image::new(ctx, "/arrow.png")?,
            draw_state: DrawState::NotHolding,
            dead_arrows: SpriteBatch::new(graphics::Image::new(ctx, "/dead_arrow.png")?),
            dead_arrow_shadows: SpriteBatch::new(graphics::Image::new(ctx, "/dead_arrow.png")?),
            spot_mesh: create_spot_mesh(ctx)?,
        })
    }

    pub fn update_tick(&mut self) {
        for (i, mut arrow) in self.live_arrows.iter_mut().enumerate() {
            arrow.height += arrow.climb_momentum;
            if arrow.height <= 16. && arrow.climb_momentum < 0. {
                self.temp_usize.push(i);
                arrow.climb_momentum *= 3.0;
                arrow.position = arrow.position.add(arrow.momentum * 1.5);

                TODO /*
                    generalize these arrow draw param functions in main
                    insert arrow here into dead_arrow_shadows
                */

                let param = graphics::DrawParam {
                    dest: Point2::new(arrow.position[0], arrow.position[1] - arrow.height),
                    rotation: arrow.image_angle(),
                    scale: Point2::new(arrow.image_draw_length(), 1.),
                    ..Default::default()
                };
                self.dead_arrows.add(param);
            } else {
                arrow.position = arrow.position.add(arrow.momentum);
                arrow.climb_momentum -= GRAVITY; //(-1.0_f32).max(arrow.climb_momentum-0.1);
            }
        }
        for index in self.temp_usize.drain(..).rev() {
            self.live_arrows.remove(index);
        }
    }

    fn draw_point_prop(&self, prop: f32) -> Option<Point2> {
        if prop < 0.0 {
            return None;
        }
        let index = ((self.mouse_pts.len() - 1) as f32 * prop) as usize;
        self.draw_point_abs(index)
    }

    fn draw_point_abs(&self, index: usize) -> Option<Point2> {
        if index < self.mouse_pts.len() {
            return Some(self.mouse_pts[index]);
        } else {
            None
        }
    }
}

fn create_spot_mesh(ctx: &mut Context) -> GameResult<Mesh> {
    MeshBuilder::new()
        .circle(DrawMode::Fill, Point2::new(0., 0.), 5.0, 1.0)
        .build(ctx)
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.update_tick();
        }
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        if button != MouseButton::Left {
            return;
        }
        //println!("mouse down x:{} y:{}, button:{:?}", x, y, button);
        let origin = Point2::new(x as f32, y as f32);
        self.draw_state = DrawState::Nocking(origin);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        if button != MouseButton::Left {
            return;
        }

        if let DrawState::Drawing(origin, turnaround_index) = self.draw_state {
            if turnaround_index < self.mouse_pts.len() - 1 {
                let end = Point2::new(x as f32, y as f32);
                let nock = self.mouse_pts[turnaround_index];
                let len_on = origin.dist(nock);
                let len_ne = end.dist(nock);
                let len_oe = origin.dist(end);
                let pt_x = self.draw_point_prop(0.3).unwrap();
                let pt_y = self.draw_point_prop(0.6).unwrap();
                let len_xn = pt_x.dist(nock);
                let len_ny = pt_y.dist(nock);
                let len_ye = pt_y.dist(end);
                println!(
                    "Loosed arrow!. on:{} ne:{} oe:{} xn:{}",
                    len_on, len_ne, len_oe, len_xn
                );
                let power = (len_on + len_ne) / (8.0 * len_xn + len_on + len_ne);
                let l = self.mouse_pts.len() as f32;

                let umph = 30.0 * len_ne / (len_oe + len_ne + len_on);
                let t = turnaround_index as f32;

                let theta = (PI * 0.5)
                    / (1.0
                        + 5.0
                            * (
                    (t / len_on) / ((l-t) / len_ne) // >1 if after turnaround index is WINDIER
                )); // closer to Pi/2 when 2nd section of line is WINDIER than first
                    //"windiness" = length / num points

                let pitch = umph * theta.sin();
                let speed = umph * theta.cos();
                println!("umph:{}\tspeed:{}\ttheta:{}", umph, speed, theta);
                let mom = Point2::new(
                    speed * (nock[0] - end[0]) / len_ne,
                    speed * (nock[1] - end[1]) / len_ne,
                );

                let new_arrow = LiveArrow::new(self.character_at, mom, pitch);

                println!("ANGLE IS {}", mom.rotation());
                self.live_arrows.push(new_arrow);
            }
        }
        self.draw_state = DrawState::NotHolding;
        self.mouse_pts.clear();
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        match self.draw_state {
            DrawState::NotHolding => {}
            DrawState::Nocking(origin) => {
                let pt = Point2::new(x as f32, y as f32);

                if !self.mouse_pts.is_empty() {
                    let pt_index = self.mouse_pts.len() - 1;
                    let prev_pt = self.mouse_pts[pt_index];
                    let p_dist = origin.dist(prev_pt);
                    if p_dist > 30. && p_dist > origin.dist(pt) {
                        // getting closer to origin
                        self.draw_state = DrawState::Drawing(origin, pt_index);
                    }
                }
                self.mouse_pts.push(pt);
            }
            DrawState::Drawing(_origin, _turnaround_index) => {
                let pt = Point2::new(x as f32, y as f32);
                self.mouse_pts.push(pt);
            }
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        println!("key press: {:?}", keycode);
        match keycode {
            Keycode::Escape => ctx.quit().unwrap(),
            Keycode::F4 => graphics::set_fullscreen(ctx, true).unwrap(),
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);


        graphics::set_color(ctx, graphics::BLACK)?;
        graphics::draw_ex(ctx, &self.dead_arrow_shadows, default_param())?;
        graphics::set_color(ctx, graphics::WHITE)?;
        graphics::draw_ex(ctx, &self.dead_arrows, default_param())?;

        //arrow shadows
        graphics::set_color(ctx, graphics::BLACK)?;
        for arrow in self.live_arrows.iter() {
            let param = graphics::DrawParam {
                dest: arrow.position,
                rotation: arrow.angle,
                scale: Point2::new(arrow.shadow_draw_length(), 1.0),
                ..Default::default()
            };
            graphics::draw_ex(ctx, &self.arrow_graphic, param)?;
        }

        //arrows
        graphics::set_color(ctx, graphics::WHITE)?;
        for arrow in self.live_arrows.iter() {
            let param = graphics::DrawParam {
                dest: Point2::new(arrow.position[0], arrow.position[1] - arrow.height),
                rotation: arrow.image_angle(),
                scale: Point2::new(arrow.image_draw_length(), 1.),
                ..Default::default()
            };
            graphics::draw_ex(ctx, &self.arrow_graphic, param)?;
        }

        //ui
        if self.mouse_pts.len() > 1 {
            match self.draw_state {
                DrawState::NotHolding => {
                    //pass
                }
                DrawState::Nocking(_origin) => {
                    graphics::set_color(ctx, utils::red())?;
                    graphics::line(ctx, &self.mouse_pts, 3.0)?;
                }
                DrawState::Drawing(_origin, turnaround_index) => {
                    graphics::set_color(ctx, utils::red())?;
                    graphics::line(ctx, &self.mouse_pts[..turnaround_index], 3.0)?;
                    graphics::set_color(ctx, utils::green())?;
                    graphics::line(ctx, &self.mouse_pts[turnaround_index - 1..], 3.0)?;

                    graphics::set_color(ctx, graphics::WHITE)?;
                    if let Some(point) = self.draw_point_prop(0.3) {
                        let param = graphics::DrawParam {
                            dest: point,
                            ..Default::default()
                        };
                        graphics::draw_ex(ctx, &self.spot_mesh, param)?;
                    }
                    if let Some(point) = self.draw_point_prop(0.6) {
                        graphics::draw(ctx, &self.spot_mesh, point, 0.0)?;
                    }
                }
            }
        }

        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }
}

fn main() {
    let c = conf::Conf::new();
    let mut ctx = &mut Context::load_from_conf("bowdraw", "ggez", c).unwrap();
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }
    let mut gs = GameState::new(&mut ctx).expect("err making GameState");
    event::run(ctx, &mut gs).unwrap();
}

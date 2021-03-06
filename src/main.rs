extern crate ggez;
extern crate rand;

mod consts;
mod synchro;
#[macro_use]
mod utils;
mod arrow;
mod assets;

use arrow::LiveArrow;
use assets::{Assets, SourceExtension};
use consts::*;
use std::f32::consts::PI;
use utils::PointArithmetic;

use ggez::{
    audio, conf,
    event::{self, Keycode, Mod, MouseButton, MouseState},
    graphics::{self, spritebatch::SpriteBatch, DrawMode, DrawParam, Mesh, MeshBuilder, Point2},
    timer, Context, GameResult,
};
use std::{
    env, path,
    time::{Duration, Instant},
};

enum DrawState {
    NotHolding,             // not drawn back
    Nocking(Point2, bool),  // origin, clicked
    Drawing(Point2, usize), // origin, turnaround_index
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DiscreteNotch {
    Pos,
    Neg,
    Zero,
}
impl DiscreteNotch {
    pub fn reset(&mut self) {
        *self = DiscreteNotch::Zero;
    }
    pub fn mul(self, value: f32) -> f32 {
        use DiscreteNotch::*;
        match self {
            Neg => -value,
            Zero => 0.,
            Pos => value,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum AxiomaticDirection {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Copy, Clone, Debug)]
struct PressingState {
    vertical: DiscreteNotch,
    horizontal: DiscreteNotch,
}
impl PressingState {
    pub fn new() -> Self {
        PressingState {
            vertical: DiscreteNotch::Zero,
            horizontal: DiscreteNotch::Zero,
        }
    }
    pub fn is_all_zero(&self) -> bool {
        use DiscreteNotch::*;
        if let (Zero, Zero) = (self.vertical, self.horizontal) {
            true
        } else {
            false
        }
    }
}

struct InputConfig {
    up: Keycode,
    down: Keycode,
    left: Keycode,
    right: Keycode,
    double_tap_window: Duration,
}

struct GameState {
    character_at: Point2,
    live_arrows: Vec<LiveArrow>,
    temp_usize: Vec<usize>,
    // arrow_graphic: graphics::Image,
    mouse_pts: Vec<Point2>,
    draw_state: DrawState,
    spot_mesh: Mesh,
    dead_arrows: SpriteBatch,
    dead_arrow_shadows: SpriteBatch,
    pressing_state: PressingState,
    input_config: InputConfig,
    assets: Assets,
    last_move_tap: (Instant, AxiomaticDirection),
}

/////////////////////////////////////////////////////////////////////////////

impl GameState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let assets = Assets::new(ctx)?;
        Ok(Self {
            live_arrows: vec![],
            temp_usize: vec![],
            character_at: Point2::new(400., 300.),
            mouse_pts: vec![],
            draw_state: DrawState::NotHolding,
            dead_arrows: SpriteBatch::new(assets.i.dead_arrow.clone()),
            dead_arrow_shadows: SpriteBatch::new(assets.i.dead_arrow.clone()),
            spot_mesh: Self::create_spot_mesh(ctx)?,
            pressing_state: PressingState::new(),
            input_config: InputConfig {
                up: Keycode::W,
                down: Keycode::S,
                left: Keycode::A,
                right: Keycode::D,
                double_tap_window: Duration::from_millis(250),
            },
            assets,
            last_move_tap: (Instant::now(), AxiomaticDirection::Up),
        })
    }

    fn param_image_arrow(arrow: &LiveArrow) -> DrawParam {
        DrawParam {
            dest: Point2::new(arrow.position[0], arrow.position[1] - arrow.height),
            rotation: arrow.image_angle(),
            offset: Point2::new(1.0, 0.5),
            scale: Point2::new(arrow.image_draw_length(), 1.),
            color: Some(ggez::graphics::WHITE),
            ..Default::default()
        }
    }

    fn param_shadow_arrow(arrow: &LiveArrow) -> DrawParam {
        DrawParam {
            dest: arrow.position,
            offset: Point2::new(1.0, 0.5),
            rotation: arrow.angle,
            scale: Point2::new(arrow.shadow_draw_length(), 1.0),
            color: Some(ggez::graphics::BLACK),
            ..Default::default()
        }
    }

    pub fn update_tick(&mut self, ctx: &mut Context) {
        self.update_live_arrows(ctx);
        self.update_character_pos(ctx);
    }

    fn update_character_pos(&mut self, ctx: &mut Context) {
        if !self.pressing_state.is_all_zero() {
            use DiscreteNotch::*;
            let speed =
                if self.pressing_state.vertical == Zero || self.pressing_state.horizontal == Zero {
                    CHAR_MOV_SPEED
                } else {
                    CHAR_MOV_SPEED * (2.0_f32).sqrt()
                };
            let char_offset = Point2::new(
                self.pressing_state.horizontal.mul(speed),
                self.pressing_state.vertical.mul(speed),
            );
            self.character_at = self.character_at.add(char_offset);
        }
    }

    fn update_live_arrows(&mut self, ctx: &mut Context) {
        // update arr
        for (i, mut arrow) in self.live_arrows.iter_mut().enumerate() {
            arrow.momentum *= ARROW_DRAG_MULT;;
            arrow.height += arrow.climb_momentum;
            if arrow.height <= 0. && arrow.climb_momentum < 0. {
                self.temp_usize.push(i);
                arrow.momentum *= 0.5;
                arrow.position = arrow.position.add(arrow.momentum * 1.5);
                self.assets.a.arrowthud.play().unwrap();
                self.dead_arrows.add(Self::param_image_arrow(arrow));
                self.dead_arrow_shadows.add(Self::param_shadow_arrow(arrow));
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

    // trigger each time you press a move direction key
    // returns if it was a DOUBLE TAP
    fn dir_tap(&mut self, dir: AxiomaticDirection) -> bool {
        let now = Instant::now();
        let double_tapped = if now - self.last_move_tap.0 <= self.input_config.double_tap_window
            && self.last_move_tap.1 == dir
        {
            println!("double tap in dir: {:?}", dir);
            true
        } else {
            false
        };
        self.last_move_tap = (now, dir);
        double_tapped
    }

    fn create_spot_mesh(ctx: &mut Context) -> GameResult<Mesh> {
        MeshBuilder::new()
            .circle(DrawMode::Fill, Point2::new(0., 0.), 5.0, 1.0)
            .build(ctx)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.update_tick(ctx);
        }
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        if button != MouseButton::Left {
            return;
        }
        //println!("mouse down x:{} y:{}, button:{:?}", x, y, button);
        let origin = Point2::new(x as f32, y as f32);
        self.draw_state = DrawState::Nocking(origin, false);
        self.assets.a.quiverdraw.play().unwrap();
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        if button != MouseButton::Left {
            return;
        }

        if let DrawState::Drawing(origin, turnaround_index) = self.draw_state {
            // Release an arrow!

            if turnaround_index < self.mouse_pts.len() - 1 {
                let end = Point2::new(x as f32, y as f32);
                let nock = self.mouse_pts[turnaround_index];
                let len_on = origin.dist(nock);
                let len_ne = end.dist(nock);
                let len_oe = origin.dist(end);
                let l = self.mouse_pts.len() as f32;

                let umph = ARROW_BASE_UMPH * len_ne / (len_oe + len_ne + len_on);
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
                let x = self.assets.a.bowshot.play();
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
            DrawState::Nocking(origin, clicked) => {
                let pt = Point2::new(x as f32, y as f32);

                if !self.mouse_pts.is_empty() {
                    let pt_index = self.mouse_pts.len() - 1;
                    let prev_pt = self.mouse_pts[pt_index];
                    let p_dist = origin.dist(prev_pt);

                    if p_dist > 50. {
                        if !clicked {
                            self.draw_state = DrawState::Nocking(origin, true);
                            self.assets.a.nock.play().unwrap();
                        }
                        if p_dist > origin.dist(pt) {
                            // getting closer to origin. switch to DRAWING
                            self.draw_state = DrawState::Drawing(origin, pt_index);
                        }
                    }
                }
                self.mouse_pts.push(pt);
            }
            DrawState::Drawing(origin, turnaround_index) => {
                let pt = Point2::new(x as f32, y as f32);
                let nockpt = self.mouse_pts[turnaround_index];
                let nocklen = nockpt.dist(origin);
                let drawlen = nockpt.dist(pt);
                self.assets
                    .a
                    .bowdraw
                    .set_volume(drawlen / (nocklen + drawlen));
                self.assets.a.bowdraw.play_if_not_playing().unwrap();
                let pt = Point2::new(x as f32, y as f32);
                self.mouse_pts.push(pt);
            }
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        println!("key press: {:?}", keycode);
        match keycode {
            x if x == self.input_config.up => {
                if let DiscreteNotch::Neg = self.pressing_state.vertical {
                    self.pressing_state.vertical.reset()
                }
            }
            x if x == self.input_config.down => {
                if let DiscreteNotch::Pos = self.pressing_state.vertical {
                    self.pressing_state.vertical.reset()
                }
            }
            x if x == self.input_config.left => {
                if let DiscreteNotch::Neg = self.pressing_state.horizontal {
                    self.pressing_state.horizontal.reset()
                }
            }
            x if x == self.input_config.right => {
                if let DiscreteNotch::Pos = self.pressing_state.horizontal {
                    self.pressing_state.horizontal.reset()
                }
            }
            _ => (),
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, repeat: bool) {
        if repeat {
            return;
        }
        println!("key press: {:?}", keycode);
        match keycode {
            Keycode::Escape => ctx.quit().unwrap(),
            Keycode::F4 => {
                let was = graphics::is_fullscreen(ctx);
                graphics::set_fullscreen(ctx, !was).expect("failed to set fullscreen");
            }
            x if x == self.input_config.up => {
                self.pressing_state.vertical = DiscreteNotch::Neg;
                self.dir_tap(AxiomaticDirection::Up);
            }
            x if x == self.input_config.down => {
                self.pressing_state.vertical = DiscreteNotch::Pos;
                self.dir_tap(AxiomaticDirection::Down);
            }
            x if x == self.input_config.left => {
                self.pressing_state.horizontal = DiscreteNotch::Neg;
                self.dir_tap(AxiomaticDirection::Left);
            }
            x if x == self.input_config.right => {
                self.pressing_state.horizontal = DiscreteNotch::Pos;
                self.dir_tap(AxiomaticDirection::Right);
            }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // println!("playing: {:?}", self.sound_effects.draw.playing());
        //dead arrows
        graphics::clear(ctx);
        graphics::set_color(ctx, graphics::WHITE)?;
        graphics::draw_ex(ctx, &self.dead_arrow_shadows, DrawParam::default())?;
        graphics::draw_ex(ctx, &self.dead_arrows, DrawParam::default())?;

        //live arrows
        for param in self.live_arrows.iter().map(Self::param_shadow_arrow) {
            graphics::draw_ex(ctx, &self.assets.i.arrow, param)?;
        }
        for param in self.live_arrows.iter().map(Self::param_image_arrow) {
            graphics::draw_ex(ctx, &self.assets.i.arrow, param)?;
        }

        //character
        graphics::set_color(ctx, utils::blue())?;
        graphics::draw(ctx, &self.spot_mesh, self.character_at, 0.0)?;

        //ui
        if self.mouse_pts.len() > 1 {
            match self.draw_state {
                DrawState::NotHolding => {
                    //pass
                }
                DrawState::Nocking(_origin, _clicked) => {
                    graphics::set_color(ctx, utils::red())?;
                    graphics::line(ctx, &self.mouse_pts, 3.0)?;
                }
                DrawState::Drawing(_origin, turnaround_index) => {
                    graphics::set_color(ctx, utils::red())?;
                    graphics::line(ctx, &self.mouse_pts[..turnaround_index], 3.0)?;
                    graphics::set_color(ctx, utils::green())?;
                    graphics::line(ctx, &self.mouse_pts[turnaround_index - 1..], 3.0)?;
                    graphics::set_color(ctx, utils::blue())?;
                    graphics::line(
                        ctx,
                        &[
                            self.mouse_pts[turnaround_index],
                            *self.mouse_pts.last().unwrap(),
                        ],
                        2.0,
                    )?;

                    graphics::set_color(ctx, graphics::WHITE)?;
                    if let Some(point) = self.draw_point_prop(0.3) {
                        graphics::draw(ctx, &self.spot_mesh, point, 0.0)?;
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

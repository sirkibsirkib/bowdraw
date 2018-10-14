
extern crate ggez;
extern crate fnv;
extern crate rand;

use std::{
	// thread,
	// time,
	path,
	env,
};
use ggez::{
    Context,
    GameResult,
    conf,
    timer,
    graphics::{
        self,
        Color,
        DrawMode,
        Point2,
        MeshBuilder,
        Mesh,
        spritebatch::SpriteBatch,
    },
    event::{
        self,
        Keycode,
        Mod,
        MouseState,
        MouseButton,
    },
};

// struct DrawData {
//     state: DrawState,
//     origin: Point2,
//     turnaround_index
// }

enum DrawState {
    NotHolding,
    Nocking(Point2), //origin
    Drawing(Point2, usize), //(origin, turnaround_index)
    // Loosed(Point2, usize),
}
impl DrawState {
    // fn am_holding(&self) -> bool {
    //     match self {
    //         DrawState::NotHolding => false,
    //         _ => true,
    //     }
    // }
}

const DESIRED_UPS: u32 = 60;
// const DESIRED_FPS: u32 = 60;




struct LiveArrow {
    position: Point2,
    angle: f32,
    momentum: Point2,
    height: f32,
    climb_momentum: f32, 
}
impl LiveArrow {

    fn shadow_draw_length(&self) -> f32 {
        let pi = std::f32::consts::PI;
        let x = self._vert_draw_ratio().abs();
        let y = ((x*pi).cos()+1.)/2.;
        let val = (0.7*y + 0.2);
        val*0.3*self.momentum[0].abs()/offset_pt(self.momentum) + 0.7*val
    }

    // returns 1.0 when arrow is shot straight up, -1.0 when shot straight down
    fn _vert_draw_ratio(&self) -> f32 {
        let x = self.angle.sin();
        let hyp = self.climb_momentum.hypot(offset_pt(self.momentum));
        let climb_ratio = self.climb_momentum / hyp;
        let x_influence = 0.5 + 0.5*x.abs();
        x_influence*climb_ratio
    }

    fn image_draw_length(&self) -> f32 {
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

    fn image_angle(&self) -> f32 {
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
    fn shadow_angle(&self) -> f32 {
        self.angle
    }
}

struct GameState {
    character_at: Point2,
    live_arrows: Vec<LiveArrow>,
    dead_arrows: SpriteBatch,
    temp_usize: Vec<usize>,
    arrow_graphic: graphics::Image,
    mouse_pts: Vec<Point2>,
    draw_state: DrawState,
    spot_mesh: Mesh,
}

macro_rules! sqr {
    ($x:expr) => ($x*$x)
}

fn dist(a: Point2, b: Point2) -> f32 {
    offset_pt(sub_pts(a, b))
}

fn offset_pt(p: Point2) -> f32 {
    (
        sqr!(p[0]) + sqr!(p[1])
    ).sqrt()
}

fn sub_pts(a: Point2, b: Point2) -> Point2 {
    Point2::new(a[0]-b[0], a[1]-b[1])
}

fn add_pts(a: Point2, b: Point2) -> Point2 {
    Point2::new(a[0]+b[0], a[1]+b[1])
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
            spot_mesh: create_spot_mesh(ctx)?,
		})
	}

	pub fn update_tick(&mut self) {
        for (i, mut arrow) in self.live_arrows.iter_mut().enumerate() {
            arrow.height += arrow.climb_momentum;
            if arrow.height <= 16. && arrow.climb_momentum < 0. {
                self.temp_usize.push(i);
                arrow.climb_momentum *= 3.0;
                arrow.position = add_pts(arrow.position, scale_pt(arrow.momentum, 1.5));
                let param = graphics::DrawParam {
                    dest: Point2::new(arrow.position[0], arrow.position[1]-arrow.height),
                    rotation: arrow.image_angle(),
                    scale: Point2::new(arrow.image_draw_length(), 1.),
                    ..Default::default()
                };
                self.dead_arrows.add(param);
            } else {
                arrow.position = add_pts(arrow.position, arrow.momentum);
                arrow.climb_momentum -= 0.25;//(-1.0_f32).max(arrow.climb_momentum-0.1);
            }
        }
        for index in self.temp_usize.drain(..).rev() {
            self.live_arrows.remove(index);
        }
	}

    // pub fn am_clicking(&self) -> bool {
    //     self.draw_state.am_holding()
    // }



    fn draw_point_prop(&self, prop: f32) -> Option<Point2> {
        if prop < 0.0 {return None}
        let index = ((self.mouse_pts.len()-1) as f32 * prop) as usize;
        self.draw_point_abs(index)
    }

    fn draw_point_abs(&self, index: usize) -> Option<Point2> {
        if index < self.mouse_pts.len() {
            return Some(self.mouse_pts[index]);
        } else { None }
    }
}

fn create_spot_mesh(ctx: &mut Context) -> GameResult<Mesh> {
    MeshBuilder::new()
        .circle(DrawMode::Fill, Point2::new(0., 0.), 5.0, 1.0)
        .build(ctx)
}

fn green() -> Color {Color::new(0., 1., 0., 1.)}
fn red() -> Color {Color::new(1., 0., 0., 1.)}
fn blue() -> Color {Color::new(0., 0., 1., 1.)}

fn calc_rotation(pt: Point2) -> f32 {
    pt[1].atan2(pt[0])
}

fn scale_pt(p: Point2, scalar: f32) -> Point2 {
    Point2::new(p[0]*scalar, p[1]*scalar)
}

impl event::EventHandler for GameState {
	fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, DESIRED_UPS) {
        	self.update_tick();

        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self, 
        _ctx: &mut Context, 
        button: MouseButton, 
        x: i32, 
        y: i32
    ) {
        if button != MouseButton::Left {
            return;
        }
        //println!("mouse down x:{} y:{}, button:{:?}", x, y, button);
        let origin = Point2::new(x as f32, y as f32);
        self.draw_state = DrawState::Nocking(origin);
    }

    fn mouse_button_up_event(
        &mut self, 
        _ctx: &mut Context, 
        button: MouseButton, 
        x: i32, 
        y: i32
    ) {

        if button != MouseButton::Left {
            return;
        }
        //println!("mouse up x:{} y:{}, button:{:?}", x, y, button);
        if let DrawState::Drawing(origin, turnaround_index) = self.draw_state {
            if turnaround_index < self.mouse_pts.len() - 1 {
                let end = Point2::new(x as f32, y as f32);
                let nock = self.mouse_pts[turnaround_index];
                let len_on = dist(origin, nock);
                let len_ne = dist(end, nock);
                let len_oe = dist(origin, end);
                let pt_x = self.draw_point_prop(0.3).unwrap(); 
                let pt_y = self.draw_point_prop(0.6).unwrap();  
                let len_xn = dist(pt_x, nock);  
                let len_ny = dist(pt_y, nock); 
                let len_ye = dist(pt_y, end);   
                println!("Loosed arrow!. on:{} ne:{} oe:{} xn:{}", len_on, len_ne, len_oe, len_xn);
                let power = (len_on + len_ne) / (8.0 * len_xn + len_on + len_ne);
                let pitch = 3.0 * turnaround_index as f32 / self.mouse_pts.len() as f32;
                let speed = (0.03 * len_on) / (len_oe + len_on + 1.0);
                println!("power:{} pitch:{} speed:{}", power, pitch, speed);
                
                let mom = scale_pt(Point2::new(nock[0] - end[0], nock[1] - end[1]), speed);
                println!("ANGLE IS {}", calc_rotation(mom));
                let new_arrow = LiveArrow {
                    position: self.character_at,
                    momentum: mom,
                    angle: calc_rotation(mom),
                    height: 10.0,
                    climb_momentum: pitch*3.0, 
                };
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
        _yrel: i32
    ) {
        //println!("mouse move <{},{}>  rel:<{},{}>", x, y, xrel, yrel);
        match self.draw_state {
            DrawState::NotHolding => {
                // pass
            },
            DrawState::Nocking(origin) => {
                let pt = Point2::new(x as f32, y as f32);

                if ! self.mouse_pts.is_empty() {
                    let pt_index = self.mouse_pts.len()-1;
                    let prev_pt = self.mouse_pts[pt_index];
                    let p_dist = dist(origin, prev_pt);
                    if p_dist > 30. && p_dist > dist(origin, pt) {
                        // getting closer to origin
                        self.draw_state = DrawState::Drawing(origin, pt_index);
                    }
                }
                self.mouse_pts.push(pt);
            },
            DrawState::Drawing(_origin, _turnaround_index) => {
                let pt = Point2::new(x as f32, y as f32);
                self.mouse_pts.push(pt);    
            },
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
                dest: Point2::new(arrow.position[0], arrow.position[1]-arrow.height),
                rotation: arrow.image_angle(),
                scale: Point2::new(arrow.image_draw_length(), 1.),
                ..Default::default()
            };
            graphics::draw_ex(ctx, &self.arrow_graphic, param)?;
        }
        let param = graphics::DrawParam {
            ..Default::default()
        };
        graphics::draw_ex(ctx, &self.dead_arrows, param)?;

        //ui
        if self.mouse_pts.len() > 1 {
            match self.draw_state {
                DrawState::NotHolding => {
                    //pass
                },
                DrawState::Nocking(_origin) => {
                    graphics::set_color(ctx, red())?;
                    graphics::line(ctx, &self.mouse_pts, 3.0)?;
                },
                DrawState::Drawing(_origin, turnaround_index) => {
                    graphics::set_color(ctx, red())?;
                    graphics::line(ctx, &self.mouse_pts[..turnaround_index], 3.0)?;
                    graphics::set_color(ctx, green())?;
                    graphics::line(ctx, &self.mouse_pts[turnaround_index-1..], 3.0)?;

                    graphics::set_color(ctx, graphics::WHITE)?;
                    if let Some(point) = self.draw_point_prop(0.3) {
                        let param = graphics::DrawParam {
                            dest: point,
                            ..Default::default()
                        };
                        graphics::draw_ex(ctx, &self.spot_mesh, param)?;
                    }
                    if let Some(point) = self.draw_point_prop(0.6) {
                        // let param = graphics::DrawParam {
                        //     dest: point,
                        //     ..Default::default()
                        // };
                        graphics::draw(ctx, &self.spot_mesh, point, 0.0)?;
                    }
                },
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

extern crate ggez;
extern crate fnv;
extern crate rand;

use std::{
	thread,
	time,
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

const DESIRED_UPS: u32 = 60;
const DESIRED_FPS: u32 = 60;

struct GameState {
    mouse_pts: Vec<Point2>,
    click_origin: Option<Point2>,
}

macro_rules! sqr {
    ($x:expr) => ($x*$x)
}

fn dist(a: Point2, b: Point2) -> f32 {
    (
        sqr!(a[0] - b[0]) +
        sqr!(a[1] - b[1])
    ).sqrt()
}

impl GameState {
	pub fn new(ctx: &mut Context) -> Self {
		Self {
            mouse_pts: vec![],
            click_origin: None,
		}
	}

	pub fn update_tick(&mut self) {
        //
	}

    #[inline]
    pub fn am_clicking(&self) -> bool {
        self.click_origin.is_some()
    }
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
        //println!("mouse down x:{} y:{}, button:{:?}", x, y, button);
        self.click_origin = Some(Point2::new(x as f32, y as f32));
    }

    fn mouse_button_up_event(
        &mut self, 
        _ctx: &mut Context, 
        button: MouseButton, 
        x: i32, 
        y: i32
    ) {
        //println!("mouse up x:{} y:{}, button:{:?}", x, y, button);
        self.mouse_pts.clear();
        self.click_origin = None;
    }

    fn mouse_motion_event(
        &mut self, 
        _ctx: &mut Context, 
        _state: MouseState, 
        x: i32, 
        y: i32, 
        xrel: i32, 
        yrel: i32
    ) {
        //println!("mouse move <{},{}>  rel:<{},{}>", x, y, xrel, yrel);
        if self.am_clicking() {
            self.mouse_pts.push(Point2::new(x as f32, y as f32));    
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        println!("key press: {:?}", keycode);
        match keycode {
            Keycode::Escape => ctx.quit().unwrap(),
            _ => (),
        }
    }


    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        if let Some(origin) = self.click_origin {
            let mut away = true;
            let mut last_pt = origin;
            let mut prevdist = 0.0;
            for i in 1..self.mouse_pts.len() {
                // let next = self.mouse_pts[i];
                // let d = dist(next, last_pt)
                // graphics::line(ctx, &self.mouse_pts, 3.0);
                graphics::line(ctx, &self.mouse_pts, 3.0);
            }
        }
        
        
        // let man = self.image_manager.get(ctx, ImageVariants::Man);
        // for (_player, physicality) in self.room.iter_players() {
        //     let param = graphics::DrawParam {
        //         dest: convert_coord(physicality.coord),
        //         scale: graphics::Point2::new(0.5, 0.5),
        //         ..Default::default()
        //     };
        //     graphics::draw_ex(ctx, man, param)?
        // }

        // { // draw all rocks
        //     let param = graphics::DrawParam {
        //     ..Default::default()
        //     };
        //     graphics::draw_ex(ctx, &self.batch, param)?;
        // }
        
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
    let mut gs = GameState::new(&mut ctx);
    event::run(ctx, &mut gs).unwrap();
}
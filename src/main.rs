use std::{
	thread,
	time,
};

struct Bouncer {
	pos: f32,
	vel: f32,
}
impl Bouncer {
	fn new() -> Self {
		Self { pos: 2.0, vel: 0.0 }
	}
	fn shift(&mut self) {
		self.pos += self.vel;
	}
	fn fall(&mut self) {
		self.vel += if self.pos < 0.0 {0.1} else {-0.1};
	}
}

fn main() {
    println!("Hello, world!");
    let mut bouncers : Vec<Bouncer> =  (0..4).map(|_| Bouncer::new()).collect();
    let mut i = -30;
    for (i, b) in bouncers.iter_mut().enumerate() {
    	b.pos = i as f32;
    }
    let dur = time::Duration::from_millis(200);
    loop {
    	for b in bouncers.iter_mut() {
    		b.fall();
    		b.shift();
    	}
    	show(&bouncers);
    	thread::sleep(dur);
    }
}

fn show(bouncers: &[Bouncer]) {
	for i in ((-10)..=10).rev() {
    	let i = i as f32;
    	for b in bouncers.iter() {
    		let r = b.pos - i;
    		let c = match b.pos - i {
    			x if x >= 1.00 => ' ',
    			x if x < 0.00 => ' ',
    			x if x < 0.33 => '.',
    			x if x < 0.66 => '-',
    			_             => '`',
			};
    		print!("{}  ", c);
	    }
	    println!();
    }
}
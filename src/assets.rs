
use ggez::{
    conf,
    audio::{self, Source},
    event::{self, Keycode, Mod, MouseButton, MouseState},
    graphics::{self, Image},
    timer, Context, GameResult,
};

pub struct Assets {
	pub a: AudioAssets,
	pub i: ImageAssets,
}

impl Assets {
	pub fn new(ctx: &mut Context) -> GameResult<Self> {
		Ok(Assets {
			i: ImageAssets::new(ctx)?,
			a: AudioAssets::new(ctx)?,
		})
	}
}

////////////////////////////////////////
pub struct ImageAssets {
	pub arrow: Image,
	pub dead_arrow: Image,
}

impl ImageAssets {
	fn new(ctx: &mut Context) -> GameResult<Self> {
		Ok(Self{
			arrow: Image::new(ctx, "/arrow.png")?,
			dead_arrow: Image::new(ctx, "/dead_arrow.png")?,
		})
	}
}

/////////////////////////////////
pub struct AudioAssets {
	pub bowdraw: audio::Source,	
	pub nock: audio::Source,
	pub bowshot: audio::Source,
}

impl AudioAssets {
	fn new(ctx: &mut Context) -> GameResult<Self> {
		Ok(Self{
			bowdraw: Source::new(ctx, "/longdraw.wav")?,
			nock: Source::new(ctx, "/nock.wav")?,
			bowshot: Source::new(ctx, "/bowshot.wav")?,
		})
	}
}

//////////////////////
pub trait SourceExtension {
	// returns TRUE if it wasnt already playing
	fn play_if_not_playing(&mut self) -> GameResult<bool>;
}

impl SourceExtension for Source {
	fn play_if_not_playing(&mut self) -> GameResult<bool> {
		Ok( if self.playing() { false } else {
			self.play()?;
			true
		})
	}
}

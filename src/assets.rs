use ggez::{
    audio::{self, Source},
    conf,
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
    pub mario: Image,
}

impl ImageAssets {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let mario = Image::new(ctx, "/mario.png")?;
        let mut mario_tl = split_spritesheet(ctx, &mario, [16, 16])?;
        let x = mario_tl.0.drain(..).next().unwrap();
        println!("Mario: {:?}", &x);

        Ok(Self {
            arrow: Image::new(ctx, "/arrow.png")?,
            dead_arrow: Image::new(ctx, "/dead_arrow.png")?,
            mario: x,
            // mario
        })
    }
}

/////////////////////////////////
pub struct AudioAssets {
    pub bowdraw: audio::Source,
    pub nock: audio::Source,
    pub bowshot: audio::Source,
    pub quiverdraw: audio::Source,
    pub arrowthud: audio::Source,
}

impl AudioAssets {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            bowdraw: Source::new(ctx, "/longdraw.wav")?,
            nock: Source::new(ctx, "/nock.wav")?,
            bowshot: Source::new(ctx, "/bowshot.wav")?,
            quiverdraw: Source::new(ctx, "/quiver_draw.wav")?,
            arrowthud: Source::new(ctx, "/arrow_peg.wav")?,
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
        Ok(if self.playing() {
            false
        } else {
            self.play()?;
            true
        })
    }
}

fn split_spritesheet(
    ctx: &mut Context,
    img: &Image,
    sprite_dims: [usize; 2],
) -> GameResult<(Vec<Image>, usize)> {
    let dims = [img.width() as usize, img.height() as usize];
    if dims[0] % sprite_dims[0] > 0 || dims[1] % sprite_dims[1] > 0 {
        panic!(
            "Cannot grab sprites of dim {:?} for image of dim {:?}!",
            sprite_dims, img
        );
    }
    let sprite_data = img.to_rgba8(ctx)?;
    println!("SPRITE DATA {:?}", &sprite_data);
    let mut bytebuf = Vec::new();
    let mut imgs_vec = Vec::with_capacity(sprite_dims[0] * sprite_dims[1] * 4);
    let matrix_dims = [dims[0] / sprite_dims[0], dims[1] / sprite_dims[1]];

    for j in 0..matrix_dims[1] {
        let y1 = sprite_dims[1] * j;
        let height = sprite_dims[1].min(dims[1] - j);

        for i in 0..matrix_dims[0] {
            let x1 = sprite_dims[0] * i;
            let width = sprite_dims[0].min(dims[0] - i);

            for j in 0..height {
                let start = 4 * (y1 + j) * dims[0] + x1;
                let slice = sprite_data[start..start + (4 * width)].iter().cloned();

                for byte in slice.clone() {
                    print!("{:x}", byte)
                }
                println!();
                bytebuf.extend(slice);
            }
            // println!("img contents: {:X}", &bytebuf[..]);
            imgs_vec.push(Image::from_rgba8(
                ctx,
                width as u16,
                height as u16,
                &bytebuf[..],
            )?);
            bytebuf.clear();
        }
    }
    Ok((imgs_vec, matrix_dims[0]))
}

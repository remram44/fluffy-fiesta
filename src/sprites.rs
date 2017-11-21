use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::rc::{Rc, Weak};

use graphics::ImageSize;
use graphics::color::gamma_srgb_to_linear;
use image::{self, Pixel};
use opengl_graphics::{Texture, TextureSettings};

pub struct SpriteSheet {
    pub texture: Texture,
    width: usize,
    height: usize,
}

impl SpriteSheet {
    fn from_file(name: &str) -> Result<SpriteSheet, String> {
        let filename = Path::new("assets").join(name);
        let mut image = try!(
            image::open(&filename)
            .map_err(|e| e.description().to_owned())
        ).to_rgba();
        for (_, _, pixel) in image.enumerate_pixels_mut() {
            let (r, g, b, a) = pixel.channels4();
            let r = r as f32 / 255.0;
            let g = g as f32 / 255.0;
            let b = b as f32 / 255.0;
            let a = a as f32 / 255.0;
            let new_color = gamma_srgb_to_linear([r, g, b, a]);
            let r = (new_color[0] * 255.0) as u8;
            let g = (new_color[1] * 255.0) as u8;
            let b = (new_color[2] * 255.0) as u8;
            let a = (new_color[3] * 255.0) as u8;
            *pixel = image::Pixel::from_channels(r, g, b, a);
        }

        let texture =
            Texture::from_image(&image,
                                &TextureSettings::new());
        let width = texture.get_size().0 as usize;
        let height = texture.get_size().1 as usize;
        Ok(SpriteSheet {
            texture: texture,
            width: width,
            height: height,
        })
    }
}

pub struct Sprite {
    pub sheet: Rc<SpriteSheet>,
    pub coords: [f64; 4],
    pub size: [f64; 2],
}

pub struct SpriteManager {
    sprites: RefCell<HashMap<String, Weak<SpriteSheet>>>,
}

impl SpriteManager {
    pub fn new() -> SpriteManager {
        SpriteManager {
            sprites: RefCell::new(HashMap::new()),
        }
    }

    pub fn load(&self, name: &str) -> Rc<SpriteSheet> {
        if let Some(sheet) = self.sprites.borrow().get(name) {
            if let Some(sheet) = sheet.upgrade() {
                return sheet;
            } else {
                self.sprites.borrow_mut().remove(name);
            }
        }

        let sheet = Rc::new(SpriteSheet::from_file(name).unwrap());
        self.sprites.borrow_mut().insert(name.to_owned(), Rc::downgrade(&sheet));
        sheet
    }
}

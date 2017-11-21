use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::{Rc, Weak};

use graphics::ImageSize;
use opengl_graphics::{Texture, TextureSettings};

pub struct SpriteSheet {
    pub texture: Texture,
    width: usize,
    height: usize,
}

impl SpriteSheet {
    fn from_file(name: &str) -> Result<SpriteSheet, String> {
        let texture = try!(
            Texture::from_path(&Path::new("assets").join(name),
                               &TextureSettings::new()));
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

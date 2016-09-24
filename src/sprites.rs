use piston_window;
use piston_window::texture::ImageSize;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::{Rc, Weak};

use ::Window;

pub struct SpriteSheet {
    pub texture: piston_window::G2dTexture<'static>,
    width: usize,
    height: usize,
}

impl SpriteSheet {
    fn from_file(window: &mut Window, name: &str) -> Result<SpriteSheet, String> {
        let texture = try!(
            piston_window::Texture::from_path(
                &mut window.factory,
                &Path::new("assets").join(name),
                piston_window::Flip::None,
                &piston_window::TextureSettings::new()));
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
    pub coords: [i32; 4],
    pub size: [f32; 2],
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

    pub fn load(&self, window: &mut Window, name: &str) -> Rc<SpriteSheet> {
        if let Some(sheet) = self.sprites.borrow().get(name) {
            if let Some(sheet) = sheet.upgrade() {
                return sheet;
            } else {
                self.sprites.borrow_mut().remove(name);
            }
        }

        let sheet = Rc::new(SpriteSheet::from_file(window, name).unwrap());
        self.sprites.borrow_mut().insert(name.to_owned(), Rc::downgrade(&sheet));
        sheet
    }
}

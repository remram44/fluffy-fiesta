use piston;
use piston::input::*;
use piston::window::AdvancedWindow;
use piston_window::{Context, G2d};

use std::fmt::{self, Debug, Formatter};

use ::{GameState, Resources, StateTransition};
use ::map;

mod pausemenu;

pub struct Game {
    pub map: map::Map,
}

impl Game {
    pub fn new(resources: &mut Resources) -> Game {
        Game {
            map: map::MapFactory::example().create(42),
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Game")
    }
}

impl GameState for Game {
    fn handle_event(&mut self, event: &piston::input::Event<piston::input::Input>,
                    resources: &mut Resources) -> StateTransition
    {
        if let Some(Button::Mouse(button)) = event.press_args() {
            info!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            info!("Pressed key '{:?}'", key);
            if key == Key::Escape {
                return StateTransition::Push(Box::new(pausemenu::PauseMenu::new(resources)));
            }
        }

        StateTransition::Continue
    }

    fn update(&mut self, dt: f64) -> StateTransition {
        // Drain all entities to avoid multiple

        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        // TODO: graphics
    }

    fn pause(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(false);
    }

    fn resume(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(true);
    }
}

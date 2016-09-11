use piston;
use piston::input::*;
use piston::window::AdvancedWindow;
use piston_window::{Context, G2d};

use std::fmt::{self, Debug, Formatter};

use ::{GameState, Resources, StateTransition};

mod pausemenu;

pub struct Game {
    rotation: f64,  // Rotation for the square.
    capture: bool,
}

impl Game {
    pub fn new(resources: &mut Resources) -> Game {
        Game {
            rotation: 0.0,
            capture: false,
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
            } else if key == Key::C {
                self.capture = !self.capture;
                resources.window.set_capture_cursor(self.capture);
                info!("capture {}", if self.capture { "on" } else { "off" });
            }
        }

        StateTransition::Continue
    }

    fn update(&mut self, dt: f64) -> StateTransition {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * dt;

        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 150.0);
        let rotation = self.rotation;
        let (x, y) = (100.0, 100.0);

        // Clear the screen.
        clear(GREEN, g);

        let transform = c.transform.trans(x, y)
                                   .rot_rad(rotation)
                                   .trans(-75.0, -75.0);

        // Draw a box rotating around the middle of the screen.
        rectangle(RED, square, transform, g);
    }

    fn pause(&mut self, resources: &mut Resources) {
        if self.capture {
            resources.window.set_capture_cursor(false);
        }
    }

    fn resume(&mut self, resources: &mut Resources) {
        if self.capture {
            resources.window.set_capture_cursor(self.capture);
        }
    }
}

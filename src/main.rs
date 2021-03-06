#[macro_use] extern crate conrod;
extern crate env_logger;
extern crate gfx_core;
extern crate graphics;
#[macro_use] extern crate log;
extern crate piston;
extern crate piston_window;
extern crate sdl2_window;
extern crate vecmath as vecmath_lib;

use std::fmt::Debug;
use std::rc::Rc;

use gfx_core::Device;
use piston::window::WindowSettings;
use piston_window::{Context, G2d, OpenGL, PistonWindow};
use piston::input::*;
use sdl2_window::Sdl2Window;

mod entities;
mod game;
mod input;
mod sprites;
mod utils;
mod vecmath;
mod world;

use input::InputManager;
use sprites::{SpriteManager, SpriteSheet};

type Window = PistonWindow<Sdl2Window>;

/// A transition requested by a game state.
pub enum StateTransition {
    /// Stay on this state.
    Continue,
    /// End this state and pop it from the stack.
    End,
    /// Pop this state and replace it with this new state.
    Replace(Box<GameState>),
    /// Keep this state, but pause it and push the state on top of the stack.
    Push(Box<GameState>),
    /// Quit the game completely.
    Quit,
}

/// A state the game can be in.
///
/// The application consists of a stack of states, that can each push other
/// states on the stack and/or pop themselves.
/// Examples of states are the main menu, the character selection, or the game
/// itself; starting the game finishes the character selection, but the main
/// menu remains accessible.
pub trait GameState: Debug {
    fn handle_event(&mut self, event: &piston::input::Event,
                    resources: &mut Resources) -> StateTransition
    {
        StateTransition::Continue
    }
    fn update(&mut self, dt: f64, resources: &mut Resources) -> StateTransition;
    fn draw(&mut self, c: Context, g: &mut G2d);
    fn pause(&mut self, resources: &mut Resources) {}
    fn resume(&mut self, resources: &mut Resources) {}
}

pub struct Resources {
    window: Window,
    input_manager: InputManager,
    sprite_manager: SpriteManager,
}

impl Resources {
    pub fn load_spritesheet(&mut self, name: &str) -> Rc<SpriteSheet> {
        self.sprite_manager.load(&mut self.window, name)
    }
}

struct App {
    states: Vec<Box<GameState>>,
    resources: Resources,
}

impl App {
    fn new() -> App {
        let width = 800;
        let height = 600;

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an SDL2 window.
        let window: Window = WindowSettings::new(
                "fluffy-fiesta",
                [width, height],
            )
            .opengl(opengl)
            .build()
            .unwrap();
        info!("Window created");

        let mut app = App {
            states: Vec::new(),
            resources: Resources {
                window: window,
                input_manager: InputManager::new(),
                sprite_manager: SpriteManager::new(),
            },
        };
        let game = game::Game::new(world::MapFactory::example(), &mut app.resources);
        app.states.push(Box::new(game));
        info!("Game state created");
        app
    }

    fn run(&mut self) {
        loop {
            let transition: StateTransition = if let Some(state) = self.states.last_mut() {
                Self::handle_state(&mut self.resources, state)
            } else {
                break
            };
            match transition {
                StateTransition::Continue => panic!("App::run() got Transition::Continue"),
                StateTransition::End => {
                    let previous = self.states.pop().expect("Transition::End with no states");
                    info!("Dropped {:?}", previous);
                }
                StateTransition::Replace(state) => {
                    let previous = self.states.pop().expect("Transition::Replace with no states");
                    info!("Dropped {:?}", previous);
                    info!("Created {:?}", state);
                    self.states.push(state);
                }
                StateTransition::Push(state) => {
                    info!("Created {:?}", state);
                    self.states.push(state);
                }
                StateTransition::Quit => {
                    info!("Exiting...");
                    break
                },
            }
        }
    }

    fn handle_state(resources: &mut Resources, state: &mut Box<GameState>) -> StateTransition {
        info!("Executing {:?}", state);
        state.resume(resources);

        while let Some(event) = resources.window.next() {
            // Handle generic event
            let transition = state.handle_event(&event, resources);
            match transition {
                StateTransition::Continue => {},
                t => {
                    state.pause(resources);
                    return t;
                }
            }

            // Call update method
            if let Some(u) = event.update_args() {
                let transition = state.update(u.dt, resources);
                match transition {
                    StateTransition::Continue => {},
                    t => {
                        state.pause(resources);
                        return t;
                    }
                }
            }

            // Call draw method
            if let Some(r) = event.render_args() {
                resources.window.draw_2d(&event, |c, g| state.draw(c, g));
                resources.window.device.cleanup();
            }
        }
        info!("Stopping {:?}", state);
        state.pause(resources);
        StateTransition::End
    }
}

fn main() {
    env_logger::init().unwrap();
    info!("Starting up...");
    let mut app = App::new();
    info!("Running application");
    app.run();
}

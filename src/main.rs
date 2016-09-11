#[macro_use] extern crate conrod;
extern crate gfx_device_gl;
extern crate graphics;
extern crate piston;
extern crate piston_window;
extern crate sdl2_window;
extern crate vecmath as vecmath_lib;

use std::path::Path;

use conrod::{Labelable, Positionable, Sizeable, Widget};
use piston::window::{AdvancedWindow, WindowSettings};
use piston_window::{Context, G2d, OpenGL, PistonWindow};
use piston::event_loop::*;
use piston::input::*;
use sdl2_window::Sdl2Window;

mod map;
mod vecmath;

/// A transition requested by a game state.
enum StateTransition {
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
trait GameState {
    fn handle_event(&mut self, event: &piston::input::Event<piston::input::Input>,
                    resources: &mut Resources) -> StateTransition
    {
        StateTransition::Continue
    }
    fn update(&mut self, dt: f64) -> StateTransition;
    fn draw(&mut self, c: Context, g: &mut G2d);
    fn pause(&mut self, resources: &mut Resources) {}
    fn resume(&mut self, resources: &mut Resources) {}
}

struct Resources {
    window: PistonWindow<Sdl2Window>,
    width: u32,
    height: u32,
}

struct App {
    states: Vec<Box<GameState>>,
    resources: Resources,
}

impl App {
    fn new() -> App {
        let width = 200;
        let height = 200;

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an SDL2 window.
        let window: PistonWindow<Sdl2Window> = WindowSettings::new(
                "fluffy-fiesta",
                [width, height]
            )
            .opengl(opengl)
            .build()
            .unwrap();

        let mut app = App {
            states: Vec::new(),
            resources: Resources {
                window: window,
            width: width,
            height: height,
            },
        };
        let game = Game::new(&mut app.resources);
        app.states.push(Box::new(game));
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
                    self.states.pop().expect("Transition::End with no states");
                }
                StateTransition::Replace(state) => {
                    self.states.pop().expect("Transition::Replace with no states");
                    self.states.push(state);
                }
                StateTransition::Push(state) => {
                    self.states.push(state);
                }
                StateTransition::Quit => break,
            }
        }
    }

    fn handle_state(resources: &mut Resources, state: &mut Box<GameState>) -> StateTransition {
        state.resume(resources);

        let mut events = resources.window.events();
        while let Some(event) = events.next(&mut resources.window) {
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
                let transition = state.update(u.dt);
                match transition {
                    StateTransition::Continue => {},
                    t => {
                        state.pause(resources);
                        return t;
                    }
                }
            }

            // Call draw method
            resources.window.draw_2d(&event, |c, g| state.draw(c, g));
        }
        state.pause(resources);
        StateTransition::End
    }
}

fn main() {
    let mut app = App::new();
    app.run();
}

struct Game {
    rotation: f64,  // Rotation for the square.
    capture: bool,
}

impl Game {
    fn new(resources: &mut Resources) -> Game {
        Game {
            rotation: 0.0,
            capture: false,
        }
    }
}

impl GameState for Game {
    fn handle_event(&mut self, event: &piston::input::Event<piston::input::Input>,
                    resources: &mut Resources) -> StateTransition
    {
        if let Some(Button::Mouse(button)) = event.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            println!("Pressed key '{:?}'", key);
            if key == Key::Escape {
                println!("ui on");
                return StateTransition::Push(Box::new(PauseMenu::new(resources)));
            } else if key == Key::C {
                self.capture = !self.capture;
                resources.window.set_capture_cursor(self.capture);
                println!("capture {}", if self.capture { "on" } else { "off" });
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

widget_ids!(struct GameWidgetIds { canvas, resume, quit });

struct PauseMenu {
    ui: conrod::Ui,
    widget_ids: GameWidgetIds,
    // FIXME: Spelling out gfx_device_gl (and direct dep) shouldn't be necessary
    image_map: conrod::image::Map<piston_window::Texture<gfx_device_gl::Resources>>,
    text_texture_cache: conrod::backend::piston_window::GlyphCache,
}

impl PauseMenu {
    fn new(resources: &mut Resources) -> PauseMenu {
        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new().build();

        // Generate the widget identifiers.
        let ids = GameWidgetIds::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let font_path = Path::new("assets/NotoSans-Regular.ttf");
        assert!(font_path.exists());
        ui.fonts.insert_from_file(font_path).unwrap();

        // Create a texture to use for efficiently caching text on the GPU.
        let text_texture_cache =
            conrod::backend::piston_window::GlyphCache::new(&mut resources.window,
                                                            resources.width,
                                                            resources.height);

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::new();

        PauseMenu {
            ui: ui,
            widget_ids: ids,
            image_map: image_map,
            text_texture_cache: text_texture_cache,
        }
    }
}

impl GameState for PauseMenu {
    fn handle_event(&mut self, event: &piston::input::Event<piston::input::Input>,
                    resources: &mut Resources) -> StateTransition
    {
        // Convert the piston event to a conrod event.
        if let Some(ce) = conrod::backend::piston_window::convert_event(event.clone(), &mut resources.window) {
            self.ui.handle_event(ce);
        }

        let ui = &mut self.ui.set_widgets();

        // Create a background canvas upon which we'll place the button.
        conrod::widget::Canvas::new().floating(true).w_h(100.0, 70.0).pad(10.0).middle()
            .set(self.widget_ids.canvas, ui);

        // Draw the buttons.
        if conrod::widget::Button::new()
            .mid_top_of(self.widget_ids.canvas)
            .w_h(80.0, 20.0)
            .label("Resume")
            .set(self.widget_ids.resume, ui)
            .was_clicked()
        {
            StateTransition::End
        } else if conrod::widget::Button::new()
            .down(10.0)
            .w_h(80.0, 20.0)
            .label("Quit")
            .set(self.widget_ids.quit, ui)
            .was_clicked()
        {
            StateTransition::Quit
        } else {
            StateTransition::Continue
        }
    }

    fn update(&mut self, df: f64) -> StateTransition {
        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        let primitives = self.ui.draw();
        fn texture_from_image<T>(img: &T) -> &T { img };
        conrod::backend::piston_window::draw(c, g, primitives,
                                             &mut self.text_texture_cache,
                                             &self.image_map,
                                             texture_from_image);
    }
}

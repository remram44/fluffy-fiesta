extern crate piston;
extern crate piston_window;
extern crate graphics;
extern crate glutin_window;

use piston::window::{AdvancedWindow, WindowSettings};
use piston_window::{Context, G2d, OpenGL, PistonWindow};
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;

pub struct App {
    rotation: f64,  // Rotation for the square.
}

impl App {
    fn render(&mut self, c: &Context, g: &mut G2d) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (100.0, 100.0);

        // Clear the screen.
        clear(GREEN, g);

        let transform = c.transform.trans(x, y)
                                   .rot_rad(rotation)
                                   .trans(-25.0, -25.0);

        // Draw a box rotating around the middle of the screen.
        rectangle(RED, square, transform, g);
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: PistonWindow<GlutinWindow> = WindowSettings::new(
            "fluffy-fiesta",
            [200, 200]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        rotation: 0.0,
    };

    let mut capture = false;

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        // Draw the app and the ui.
        window.draw_2d(&e, |c, g| {
            app.render(&c, g);
        });

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            println!("Pressed key '{:?}'", key);
            if key == Key::C {
                capture = !capture;
                window.set_capture_cursor(capture);
                println!("capture {}", if capture { "on" } else { "off" });
            }
        }
    }
}

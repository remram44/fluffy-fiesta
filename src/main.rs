#[macro_use] extern crate conrod;
extern crate glutin_window;
extern crate graphics;
extern crate piston;
extern crate piston_window;
extern crate vecmath as vecmath_lib;

use std::path::Path;

use piston::window::{AdvancedWindow, WindowSettings};
use piston_window::{Context, G2d, OpenGL, PistonWindow};
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use conrod::{Labelable, Positionable, Sizeable, Widget};

mod map;
mod vecmath;

pub struct App {
    rotation: f64,  // Rotation for the square.
}

impl App {
    fn render(&mut self, c: &Context, g: &mut G2d) {
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

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

fn main() {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 200;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: PistonWindow<GlutinWindow> = WindowSettings::new(
            "fluffy-fiesta",
            [WIDTH, HEIGHT]
        )
        .opengl(opengl)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        rotation: 0.0,
    };

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { canvas, resume, quit });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let font_path = Path::new("assets/NotoSans-Regular.ttf");
    assert!(font_path.exists());
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut capture = false;
    let mut ui_on = false;

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        // Draw the app and the ui.
        window.draw_2d(&e, |c, g| {
            app.render(&c, g);

            if ui_on {
                let primitives = ui.draw();
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });

        // Convert the piston event to a conrod event.
        if let Some(ce) = conrod::backend::piston_window::convert_event(e.clone(), &window) {
            ui.handle_event(ce);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);

            if ui_on {
                let ui = &mut ui.set_widgets();

                // Create a background canvas upon which we'll place the button.
                conrod::widget::Canvas::new().floating(true).w_h(100.0, 70.0).pad(10.0).middle().set(ids.canvas, ui);

                // Draw the buttons.
                if conrod::widget::Button::new()
                    .mid_top_of(ids.canvas)
                    .w_h(80.0, 20.0)
                    .label("Resume")
                    .set(ids.resume, ui)
                    .was_clicked()
                {
                    ui_on = false;
                }

                if conrod::widget::Button::new()
                    .down(10.0)
                    .w_h(80.0, 20.0)
                    .label("Quit")
                    .set(ids.quit, ui)
                    .was_clicked()
                {
                    break;
                }
            }
        }

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            println!("Pressed key '{:?}'", key);
            if key == Key::Escape {
                println!("ui on");
                ui_on = true;
                if capture {
                    capture = false;
                    window.set_capture_cursor(false);
                }
            } else if !ui_on && key == Key::C {
                capture = !capture;
                window.set_capture_cursor(capture);
                println!("capture {}", if capture { "on" } else { "off" });
            }
        }
    }
}

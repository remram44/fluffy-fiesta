use conrod::{self, Labelable, Positionable, Sizeable, Widget};
use gfx_device_gl;
use piston;
use piston_window::{self, Context, G2d};

use std::fmt::{self, Debug, Formatter};
use std::path::Path;

use ::{GameState, Resources, StateTransition};

widget_ids!(struct GameWidgetIds { canvas, resume, quit });

pub struct PauseMenu {
    ui: conrod::Ui,
    widget_ids: GameWidgetIds,
    // FIXME: Spelling out gfx_device_gl (and direct dep) shouldn't be necessary
    image_map: conrod::image::Map<piston_window::Texture<gfx_device_gl::Resources>>,
    text_texture_cache: conrod::backend::piston_window::GlyphCache,
}

impl Debug for PauseMenu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PauseMenu")
    }
}

impl PauseMenu {
    pub fn new(resources: &mut Resources) -> PauseMenu {
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

    fn update(&mut self, dt: f64) -> StateTransition {
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

use std::fmt::{self, Debug, Formatter};
use std::path::Path;

use conrod::{self, Labelable, Positionable, Sizeable, Widget};
use piston;
use piston::window::Window;
use piston_window::{self, Context, G2d, G2dTexture};
use piston_window::texture::UpdateTexture;

use ::{GameState, Resources, StateTransition};

widget_ids!(struct GameWidgetIds { canvas, resume, quit });

pub struct PauseMenu {
    ui: conrod::Ui,
    widget_ids: GameWidgetIds,
    image_map: conrod::image::Map<G2dTexture>,
    glyph_cache: conrod::text::GlyphCache,
    text_texture_cache: G2dTexture,
    text_vertex_data: Vec<u8>,
}

impl Debug for PauseMenu {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PauseMenu")
    }
}

impl PauseMenu {
    pub fn new(resources: &mut Resources) -> PauseMenu {
        let window_size = resources.window.size();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([window_size.width as f64,
                                             window_size.height as f64])
            .build();

        // Generate the widget identifiers.
        let ids = GameWidgetIds::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let font_path = Path::new("assets/NotoSans-Regular.ttf");
        assert!(font_path.exists());
        ui.fonts.insert_from_file(font_path).unwrap();

        // Create a texture to use for efficiently caching text on the GPU.
        let (glyph_cache, text_texture_cache) = {
            let cache =
                conrod::text::GlyphCache::new(window_size.width,
                                              window_size.height,
                                              0.1, 0.1);
            let buffer_len = window_size.width as usize * window_size.height as usize;
            let init = vec![128; buffer_len];
            let settings = piston_window::TextureSettings::new();
            let factory = &mut resources.window.factory;
            let texture = G2dTexture::from_memory_alpha(
                factory, &init, window_size.width, window_size.height, &settings).unwrap();
            (cache, texture)
        };

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::new();

        PauseMenu {
            ui: ui,
            widget_ids: ids,
            image_map: image_map,
            glyph_cache: glyph_cache,
            text_texture_cache: text_texture_cache,
            text_vertex_data: Vec::new(),
        }
    }
}

impl GameState for PauseMenu {
    fn handle_event(&mut self, event: &piston::input::Input,
                    resources: &mut Resources) -> StateTransition
    {
        // Convert the piston event to a conrod event.
        let window_size = resources.window.size();
        if let Some(ce) = conrod::backend::piston::event::convert(
            event.clone(), window_size.width as f64, window_size.height as f64)
        {
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

    fn update(&mut self, dt: f64, resources: &mut Resources) -> StateTransition {
        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        let primitives = self.ui.draw();

        let text_vertex_data = &mut self.text_vertex_data;
        let cache_queued_glyphs = |graphics: &mut G2d,
                                   cache: &mut G2dTexture,
                                   rect: conrod::text::rt::Rect<u32>,
                                   data: &[u8]|
        {
            let offset = [rect.min.x, rect.min.y];
            let size = [rect.width(), rect.height()];
            let format = piston_window::texture::Format::Rgba8;
            let encoder = &mut graphics.encoder;
            text_vertex_data.clear();
            text_vertex_data.extend(
                data.iter().flat_map(|&b| vec![255, 255, 255, b]));
            UpdateTexture::update(cache, encoder, format,
                                  &text_vertex_data[..], offset, size)
                .expect("failed to update texture")
        };

        fn texture_from_image<T>(img: &T) -> &T { img };

        conrod::backend::piston::draw::primitives(primitives, c, g,
                                                  &mut self.text_texture_cache,
                                                  &mut self.glyph_cache,
                                                  &self.image_map,
                                                  cache_queued_glyphs,
                                                  texture_from_image);
    }
}

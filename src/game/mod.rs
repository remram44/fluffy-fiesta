use piston;
use piston::input::*;
use piston::window::{AdvancedWindow, Window};
use piston_window::{Context, G2d};

use std::cmp::{max, min};
use std::fmt::{self, Debug, Formatter};

use {GameState, Resources, StateTransition};
use input::InputManager;
use utils::one_rest_split_iter;
use vecmath::*;
use world::{Entity, EntityLogic, EntityPhysics, MapFactory, Spawnable, World, WorldView};

mod pausemenu;

const CAMERA_MARGIN_X: f32 = 20.0;
const CAMERA_MARGIN_Y: f32 = 20.0;

#[derive(Debug)]
struct Character {
    player: usize,
}

impl Character {
    fn new(player: usize) -> Character {
        Character {
            player: player,
        }
    }
}

impl EntityLogic for Character {
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64,
              world: &mut WorldView, input: &InputManager) -> bool {
        // Characters should be in focus
        world.focus(&entity.pos);

        // Move according to input
        if let Some(i) = input.player_input(0) {
            // Debug: fake movements
            entity.pos[0] += i.x() * dt as f32 * 5.0;
            entity.pos[1] += i.y() * dt as f32 * 5.0;
        };

        true
    }
}

struct SimpleSpawn {
    entity_logic: Option<Box<EntityLogic>>,
}

impl SimpleSpawn {
    fn new(entity_logic: Box<EntityLogic>) -> SimpleSpawn {
        SimpleSpawn {
            entity_logic: Some(entity_logic),
        }
    }
}

impl fmt::Debug for SimpleSpawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(entity_logic) = self.entity_logic.as_ref() {
            write!(f, "SimpleSpawn({:?})", entity_logic)
        } else {
            write!(f, "SimpleSpawn(empty)")
        }
    }
}

impl Spawnable for SimpleSpawn {
    fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>) {
        (false, self.entity_logic.take().map(|entity_logic| {
            Some(Entity {
                physics: EntityPhysics {
                    pos: pos.clone(),
                },
                logic: entity_logic,
            })
        }).unwrap_or(None))
    }
}
struct Camera {
    aspect_ratio: f32,
    pos: Vector2,
    size: f32,
    update_rate: f32,
}


pub struct Game {
    pub world: World,
    camera: Camera,
}

impl Game {
    pub fn new(map_factory: MapFactory, resources: &mut Resources) -> Game {
        info!("Creating game...");

        if map_factory.nb_players < 1 {
            panic!("Can't play on map meant for 0 players");
        }

        info!("Creating map");
        let mut world = map_factory.create(42);

        info!("Creating {} characters", 1);
        let character = Character::new(0);
        world.spawnables.push(Box::new(SimpleSpawn::new(Box::new(character))));

        let window_size = resources.window.size();
        let mut game = Game {
            world: world,
            camera: Camera {
                aspect_ratio: window_size.height as f32 / window_size.width as f32,
                pos: [0.0, 0.0],
                size: 10.0,
                update_rate: 1.0,
            }
        };

        // Initial update: spawns characters, set camera, ...
        game.update(0.0, resources);

        game.camera.update_rate = 0.1;
        info!("Camera: aspect_ratio = {:?}", game.camera.aspect_ratio);
        info!("Camera: Initial position: {:?}, {:?}", game.camera.pos, game.camera.size);

        game
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
        if let Some(Button::Keyboard(key)) = event.press_args() {
            if key == Key::Escape {
                return StateTransition::Push(Box::new(pausemenu::PauseMenu::new(resources)));
            }
        }

        resources.input_manager.handle_event(event);

        StateTransition::Continue
    }

    fn update(&mut self, dt: f64, resources: &mut Resources) -> StateTransition {
        resources.input_manager.update(dt);

        let map = &mut self.world.map;
        let spawnables = &mut self.world.spawnables;
        let mut focus = None;
        one_rest_split_iter(&mut self.world.entities, |mut entity, other_entities| {
            let mut world_view = WorldView {
                map: map,
                entities: other_entities,
                spawnables: spawnables,
                focus: &mut focus,
            };
            entity.logic.update(&mut entity.physics, dt, &mut world_view, &resources.input_manager);
        });
        if let Some((a, b)) = focus {
            let a = [a.x() - CAMERA_MARGIN_X, a.y() - CAMERA_MARGIN_Y];
            let b = [b.x() + CAMERA_MARGIN_X, b.y() + CAMERA_MARGIN_Y];

            let mut camera = &mut self.camera;

            // Compute desired camera position
            let ratio = camera.aspect_ratio;
            let size = (b.x() - a.x()).max((b.y() - a.y())/ratio as f32);
            let pos = [(a.x() + b.x() - size)/2.0,
                       (a.y() + b.y() - size * ratio)/2.0];

            // Update current camera position according to update rate
            camera.pos = vec2_add(vec2_scale(camera.pos, 1.0 - camera.update_rate),
                                  vec2_scale(pos, camera.update_rate));
            camera.size = camera.size * (1.0 - camera.update_rate) + size * camera.update_rate;
        }

        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        use graphics::*;

        let (width, height) = if let Some(v) = c.viewport {
            (v.rect[2], v.rect[3])
        } else {
            warn!("Got Context with no attached Viewport");
            return;
        };

        // Clear the screen.
        clear([0.0, 0.0, 0.5, 1.0], g);

        // Compute transformation from camera
        let zoom = width as f64 / self.camera.size as f64;
        let transform = c.transform
            .trans(0.0, height as f64)
            .scale(1.0, -1.0)
            .scale(zoom, zoom)
            .trans(-self.camera.pos.x() as f64, -self.camera.pos.y() as f64);

        let x1: usize = max(self.camera.pos.x() as i32 - 1, 0) as usize;
        let y1: usize = max(self.camera.pos.y() as i32 - 1, 0) as usize;
        let x2: usize = min((self.camera.pos.x() + self.camera.size + 1.0) as usize,
                            self.world.map.width);
        let y2: usize = min((self.camera.pos.y() + self.camera.size * self.camera.aspect_ratio + 1.0) as usize,
                            self.world.map.height);

        for y in y1..y2 {
            for x in x1..x2 {
                let tile = self.world.map.tile(x, y);
                rectangle(tile.color, rectangle::square(x as f64, y as f64, 1.0), transform, g);
            }
        }

        // Debug: draw grid
        for x in x1..x2 {
            rectangle([1.0, 1.0, 1.0, 1.0],
                      rectangle::centered([x as f64, (y1 + y2) as f64 * 0.5,
                                           0.5 / zoom as f64, (y2 - y1) as f64 * 0.5]),
                      transform, g);
        }
        for y in y1..y2 {
            rectangle([1.0, 1.0, 1.0, 1.0],
                      rectangle::centered([(x1 + x2) as f64 * 0.5, y as f64,
                                           (x2 - x1) as f64 * 0.5, 0.5 / zoom as f64]),
                      transform, g);
        }

        // Debug: circle entities
        for entity in self.world.entities.iter() {
            ellipse([1.0, 0.0, 0.0, 1.0],
                    rectangle::centered([entity.physics.pos.x() as f64, entity.physics.pos.y() as f64, 0.5, 0.5]),
                    transform, g);
        }
    }

    fn pause(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(false);
    }

    fn resume(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(true);
    }
}

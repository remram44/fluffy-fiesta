use piston;
use piston::input::*;
use piston::window::AdvancedWindow;
use piston_window::{Context, G2d};

use std::fmt::{self, Debug, Formatter};

use {GameState, Resources, StateTransition};
use map;
use utils::one_rest_split_iter;
use vecmath::*;

mod pausemenu;

pub struct Game {
    pub world: map::World,
}

impl Game {
    pub fn new(resources: &mut Resources) -> Game {
        use map::{Entity, EntityLogic, EntityPhysics, Spawnable, WorldView};

        info!("Creating game, adding TestSpawn");

        struct TestEntity;

        impl EntityLogic for TestEntity {
            fn update(&mut self, entity: &mut EntityPhysics, dt: f64, world: &mut WorldView) -> bool {
                info!("TestEntity updating");
                true
            }
        }

        struct TestSpawn;

        impl Spawnable for TestSpawn {
            fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>) {
                info!("TestSpawn spawning TestEntity");
                (false, Some(Entity::new(pos.clone(), TestEntity)))
            }
        }

        let mut world = map::MapFactory::example().create(42);
        world.spawnables.push(Box::new(TestSpawn));
        Game {
            world: world,
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
        let map = &mut self.world.map;
        let spawnables = &mut self.world.spawnables;
        one_rest_split_iter(&mut self.world.entities, |mut entity, other_entities| {
            let mut world_view = map::WorldView {
                map: map,
                entities: other_entities,
                spawnables: spawnables,
            };
            entity.logic.update(&mut entity.physics, dt, &mut world_view);
        });

        StateTransition::Continue
    }

    fn draw(&mut self, c: Context, g: &mut G2d) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 150.0);
        let (x, y) = (100.0, 100.0);

        // Clear the screen.
        clear(GREEN, g);

        // Draw a box around the middle of the screen.
        rectangle(RED, square, c.transform, g);

        // TODO: graphics
    }

    fn pause(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(false);
    }

    fn resume(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(true);
    }
}

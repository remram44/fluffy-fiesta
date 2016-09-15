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
        use map::{Entity, EntityLogic, Spawnable, WorldView};

        struct TestEntity;

        impl EntityLogic for TestEntity {
            fn update(&mut self, entity: &mut Entity, dt: f64, world: &mut WorldView) -> bool {
                true
            }
        }

        struct TestSpawn;

        impl Spawnable for TestSpawn {
            fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>) {
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
        one_rest_split_iter(&mut self.world.entities, |entity, other_entities| {
            let world_view = map::WorldView {
                map: map,
                entities: other_entities,
                spawnables: spawnables,
            };
        });

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

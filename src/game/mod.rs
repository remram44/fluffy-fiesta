use piston;
use piston::input::*;
use piston::window::AdvancedWindow;
use piston_window::{Context, G2d};

use std::fmt::{self, Debug, Formatter};

use map::{Entity, EntityLogic, EntityPhysics, MapFactory, Spawnable, World, WorldView};
use {GameState, Resources, StateTransition};
use utils::one_rest_split_iter;
use vecmath::*;

mod pausemenu;

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
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64, world: &mut WorldView) -> bool {
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

impl Spawnable for SimpleSpawn {
    fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>) {
        (false, self.entity_logic.take().map(|entity_logic| {
            info!("Spawning an entity");
            Some(Entity {
                physics: EntityPhysics {
                    pos: pos.clone(),
                },
                logic: entity_logic,
            })
        }).unwrap_or(None))
    }
}

pub struct Game {
    pub world: World,
}

impl Game {
    pub fn new(map_factory: MapFactory, resources: &mut Resources) -> Game {
        info!("Creating game...");

        if map_factory.nb_players < 1 {
            panic!("Can't play on map meant for 0 players");
        }

        /*
        struct TestEntity;

        impl EntityLogic for TestEntity {
            fn update(&mut self, entity: &mut EntityPhysics, dt: f64, world: &mut WorldView) -> bool {
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
        */

        info!("Creating map");
        let mut world = map_factory.create(42);

        info!("Creating {} characters", 1);
        let character = Character::new(0);
        world.spawnables.push(Box::new(SimpleSpawn::new(Box::new(character))));

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
        one_rest_split_iter(&mut self.world.entities, |mut entity, other_entities| {
            let mut world_view = WorldView {
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

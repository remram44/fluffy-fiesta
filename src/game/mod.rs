use piston;
use piston::input::*;
use piston::window::{AdvancedWindow, Window};
use piston_window::{Context, G2d};

use std::cmp::{max, min};
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

use {GameState, Resources, StateTransition};
use sprites::{Sprite, SpriteSheet};
use utils::one_rest_split_iter;
use vecmath::*;
use world::{Entity, EntityLogic, EntityPhysics, MapFactory, Spawnable, World, WorldView};

mod pausemenu;

const CAMERA_MARGIN_X: f32 = 5.0;
const CAMERA_MARGIN_Y: f32 = 5.0;

struct Character {
    player: usize,
    dir: f32,
    jump: bool,
    sprite_sheet: Rc<SpriteSheet>,
}

impl fmt::Debug for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Character for player={:?}", self.player)
    }
}

impl Character {
    fn new(player: usize, resources: &mut Resources) -> Character {
        Character {
            player: player,
            dir: 0.0,
            jump: false,
            sprite_sheet: resources.load_spritesheet(
                "alien/green__0000_idle_1.png"),
        }
    }
}

const CHAR_W: f32 = 0.63;
const CHAR_H: f32 = 1.29;
const MARGIN: f32 = 0.05;

impl EntityLogic for Character {
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64,
              world: &mut WorldView, resources: &Resources, sprite: &mut Option<Sprite>) -> bool {
        // Characters should be in focus
        world.focus(&entity.pos);

        // Read input
        if let Some(i) = resources.input_manager.player_input(0) {
            self.dir = i.x();
            self.jump = i.jump();
        };

        // Movements
        let mut on_ground = false;
        if entity.speed.y() <= 0.05 {
            if let Some(tile) = world.map.tilef(entity.pos.x(),
                                                entity.pos.y() - CHAR_H / 2. - MARGIN) {
                if tile.collide {
                    on_ground = true;
                    entity.speed[1] = 0.0;
                    entity.pos[1] = (entity.pos.y() - CHAR_H / 2. - MARGIN).floor() +
                                    1. + CHAR_H / 2.;
                }
            }
        }
        if on_ground {
            entity.speed[0] = self.dir * 5.0;
            if self.jump {
                entity.speed[1] = 5.0;
            }
        } else {
            if entity.speed.x() * self.dir.signum() < self.dir.abs() * 5.0 {
                entity.speed[0] += self.dir * 20.0 * dt as f32;
            }
            entity.speed[1] += -10.0 * dt as f32;
        }

        let dir = entity.speed.x().signum();
        for height in [-1.0f32, 1.0].iter() {
            if let Some(tile) = world.map.tilef(entity.pos.x() + (CHAR_W / 2. + MARGIN) * dir,
                                                entity.pos.y() - (CHAR_H / 2. - MARGIN) * height) {
                if tile.collide {
                    entity.pos[0] = (entity.pos.x() + (CHAR_W / 2. + MARGIN) * dir).floor() +
                        -dir * (CHAR_W / 2. + 0.5) + 0.5;
                    entity.speed[0] = 0.0;
                    break;
                }
            }
        }

        entity.pos = vec2_add(entity.pos, vec2_scale(entity.speed, dt as f32));

        // Set sprite
        // TODO: Animation
        *sprite = Some(Sprite {
            sheet: self.sprite_sheet.clone(),
            coords: [0, 0, 213, 428],
            size: [CHAR_W, CHAR_H],
        });

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
                    speed: [0.0, 0.0],
                },
                logic: entity_logic,
                sprite: None,
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
        let mut world = map_factory.create(resources, 42);

        info!("Creating {} characters", 1);
        let character = Character::new(0, resources);
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
            entity.logic.update(&mut entity.physics, dt,
                                &mut world_view, resources, &mut entity.sprite);
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

        let x1 = max(self.camera.pos.x() as i32 - 1, 0);
        let y1 = max(self.camera.pos.y() as i32 - 1, 0);
        let x2 = min((self.camera.pos.x() + self.camera.size + 1.0) as i32,
                     self.world.map.width as i32);
        let y2 = min((self.camera.pos.y() + self.camera.size * self.camera.aspect_ratio + 1.0) as i32,
                            self.world.map.height as i32);

        // Draw map
        for y in y1..y2 {
            for x in x1..x2 {
                if let Some(ref sprite) = self.world.map.tile(x, y).unwrap().sprite {
                    let image = Image::new()
                        .src_rect(sprite.coords)
                        .rect([(x as f32 + 0.5 - sprite.size[0] / 2.0) as f64,
                               (y as f32 + 0.5 + sprite.size[1] / 2.0) as f64,
                               sprite.size[0] as f64,
                               -sprite.size[1] as f64]);
                    image.draw(&sprite.sheet.texture, &DrawState::default(),
                               transform, g);
                }
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

        // Draw entities
        for entity in self.world.entities.iter() {
            if let Some(ref sprite) = entity.sprite {
                let image = Image::new()
                    .src_rect(sprite.coords)
                    .rect([(entity.physics.pos.x() - sprite.size[0] / 2.0) as f64,
                           (entity.physics.pos.y() + sprite.size[1] / 2.0) as f64,
                           sprite.size[0] as f64,
                           -sprite.size[1] as f64]);
                image.draw(&sprite.sheet.texture, &DrawState::default(),
                           transform, g);
            } else {
                // Debug: circle invisible entities
                let circle = CircleArc::new(
                    [1.0, 0.0, 0.0, 1.0],
                    0.05, 0.0, 2.0 * ::std::f64::consts::PI);
                circle.draw(
                    rectangle::centered([entity.physics.pos.x() as f64, entity.physics.pos.y() as f64, 0.5, 0.5]),
                    &DrawState::default(), transform, g);
            }
        }
    }

    fn pause(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(false);
    }

    fn resume(&mut self, resources: &mut Resources) {
        resources.window.set_capture_cursor(true);
    }
}

//! This module has the base structures for the game elements.
//!
//! The map is made of tiles, where each tile references an entry in the tile
//! type array. Tiles are ordered row first (Y goes from bottom to top) then
//! column (X goes from left to right).
//!
//! Apart from the static grid of tiles, entities represent everything else:
//! characters, items, bullets. They can occupy any decimal position, and move.
//!
//! Some tiles are active in some way, meaning that they have logic attached to
//! them. These are represented by tile entities, which means the tile types has
//! `has_entity` set to `true`, and an entity exists for each tile of that type.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use ::Resources;
use sprites::Sprite;
use vecmath::*;

/// This represents the logic for a type of entity.
pub trait EntityLogic: fmt::Debug {
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64,
              world: &mut WorldView, resources: &Resources, sprite: &mut Option<Sprite>) -> bool;
}

/// This represents the physical attributes of an entity.
pub struct EntityPhysics {
    pub pos: Vector2,
    pub speed: Vector2,
}

/// This is an entity in the world, with a position and pointer to the logic.
pub struct Entity {
    pub physics: EntityPhysics,
    pub logic: Box<EntityLogic>,
    pub sprite: Option<Sprite>,
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Entity {:?} @ {:?}", self.logic, self.physics.pos)
    }
}

type TileEntityFactory = &'static Fn(Tile, &TileType, (usize, usize)) -> Option<Entity>;

/// Definition of a tile type, referenced by tiles.
pub struct TileType {
    /// Sprite for that tile.
    pub sprite: Option<Sprite>,
    /// Damage suffered from touching that tile.
    pub damage: f64,
    /// Whether entities will collide with that tile, or pass through.
    pub collide: bool,
    /// Whether an entity is associated with this tile.
    tile_entity: bool,
}

/// A tile in the map, just references a TileType.
pub type Tile = u16;

pub trait Spawnable {
    fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>);
}

/// The map, representing the status of the world at a given instant.
pub struct Map {
    /// Width in number of tiles.
    pub width: usize,
    /// Height in number of tiles.
    pub height: usize,
    /// The tile types, referenced by the tile array.
    tiletypes: Vec<TileType>,
    /// The tiles, ordered Y first (bottom to top) then X (left to right).
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn tile(&self, x: i32, y: i32) -> Option<&TileType> {
        if x >= 0 && self.width > x as usize &&
            y >= 0 && self.height > y as usize
        {
            let tile = self.tiles[y as usize * self.width + x as usize];
            Some(&self.tiletypes[tile as usize])
        } else {
            None
        }
    }

    pub fn tilef(&self, x: f64, y: f64) -> Option<&TileType> {
        self.tile(x as i32, y as i32)
    }
}

pub struct World {
    /// The map, grid of terrain tiles.
    pub map: Map,
    /// The entities.
    pub entities: Vec<Entity>,
    /// The entities associated with tiles.
    tile_entities: HashMap<(usize, usize), Entity>,
    /// The things that can be spawned.
    pub spawnables: Vec<Box<Spawnable>>,
}

pub struct WorldView<'a> {
    pub map: &'a mut Map,
    pub entities: &'a mut Vec<Entity>,
    pub spawnables: &'a mut Vec<Box<Spawnable>>,
    pub focus: &'a mut Option<(Vector2, Vector2)>,
}

impl<'a> WorldView<'a> {
    pub fn focus(&mut self, pos: &Vector2) {
        *self.focus = Some(self.focus.map(|old| {
            ([old.0.x().min(pos.x()), old.0.y().min(pos.y())],
             [old.1.x().max(pos.x()), old.1.y().max(pos.y())])
        }).unwrap_or_else(|| { (pos.clone(), pos.clone()) }));
    }
}

struct EntityDefinition {
    type_id: String,
    position: Vector2,
}

impl EntityDefinition {
    fn create(&self, seed: u32) -> Option<Entity> {
        let logic = match self.type_id.as_ref() {
            "f.spawn" => Box::new(::entities::Spawn::new()),
            _ => {
                warn!("Can't create unknown entity type {}", self.type_id);
                return None
            },
        };
        Some(Entity {
            physics: EntityPhysics {
                pos: self.position,
                speed: [0.0, 0.0],
            },
            logic: logic,
            sprite: None,
        })
    }
}

pub struct TileTypeDefinition {
    /// Image file.
    pub sprite_sheet: &'static str,
    /// Coordinates of sprite within image file.
    pub sprite_coords: [i32; 4],
    /// Damage suffered from touching that tile.
    pub damage: f64,
    /// Whether entities will collide with that tile, or pass through.
    pub collide: bool,
    /// Factory function (creates entity).
    tile_entity: Option<Box<TileEntityFactory>>,
}

/// Initial map definition, loaded from disk.
///
/// This can be turned into a live Map using `create()`.
pub struct MapFactory {
    pub width: usize,
    pub height: usize,
    pub nb_players: usize,
    tiletypes: Vec<TileTypeDefinition>,
    tiles: Vec<Tile>,
    entities: Vec<EntityDefinition>,
}

impl MapFactory {
    /// Load a map file.
    pub fn from_file(filename: &Path) -> MapFactory {
        unimplemented!()
    }

    /// Create the hardcoded example map.
    pub fn example() -> MapFactory {
        // Initialize with background color
        let mut tiles = vec![1; 100 * 100];
        // Different background for top part
        for y in 70..100 {
            for x in 0..100 {
                tiles[y * 100 + x] = 2;
            }
        }
        // Walls all around
        for i in 0..100 {
            tiles[ 0 * 100 + i] = 0;
            tiles[99 * 100 + i] = 0;
            tiles[i * 100 +  0] = 0;
            tiles[i * 100 + 99] = 0;
        }
        // Lava at the bottom
        for x in 40..60 {
            tiles[0 * 100 + x] = 3;
        }
        for x in 0..19 {
            tiles[1 * 100 + 2 + 5 * x] = 0;
        }
        MapFactory {
            width: 100,
            height: 100,
            nb_players: 4,
            tiletypes: vec![
                // Wall
                TileTypeDefinition {
                    sprite_sheet: "map/castleCenter.png",
                    sprite_coords: [0, 0, 256, 256],
                    damage: 0.0,
                    collide: true,
                    tile_entity: None,
                },
                // Background
                TileTypeDefinition {
                    sprite_sheet: "map/bg_castle.png",
                    sprite_coords: [0, 0, 256, 256],
                    damage: 0.0,
                    collide: false,
                    tile_entity: None,
                },
                // Sky
                TileTypeDefinition {
                    sprite_sheet: "map/bg.png",
                    sprite_coords: [0, 0, 256, 256],
                    damage: 0.0,
                    collide: false,
                    tile_entity: None,
                },
                // Lava
                TileTypeDefinition {
                    sprite_sheet: "map/liquidLava.png",
                    sprite_coords: [0, 0, 256, 256],
                    damage: 1.0,
                    collide: false,
                    tile_entity: None,
                },
            ],
            tiles: tiles,
            entities: vec![
                EntityDefinition {
                    type_id: "f.spawn".to_string(),
                    position: [15.0, 1.0],
                },
                EntityDefinition {
                    type_id: "f.spawn".to_string(),
                    position: [25.0, 1.0],
                },
                EntityDefinition {
                    type_id: "f.spawn".to_string(),
                    position: [75.0, 1.0],
                },
                EntityDefinition {
                    type_id: "f.spawn".to_string(),
                    position: [85.0, 1.0],
                },
            ],
        }
    }

    /// Create a live `Map` from this map definition.
    pub fn create(&self, resources: &mut Resources, seed: u32) -> World {
        let tiletypes: Vec<TileType> = self.tiletypes.iter().map(|td| {
            TileType {
                sprite: Some(Sprite {
                    sheet: resources.load_spritesheet(td.sprite_sheet),
                    coords: td.sprite_coords,
                    size: [1.0, 1.0],
                }),
                damage: td.damage,
                collide: td.collide,
                tile_entity: td.tile_entity.is_some(),
            }
        }).collect();

        let mut tile_entities = HashMap::new();

        let tiles = self.tiles.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = tiles[y * self.width + x];
                let tiletype = &self.tiletypes[tile as usize];
                if let Some(ref factory) = tiletype.tile_entity {
                    if let Some(entity) = factory(tile, &tiletypes[tile as usize], (x, y)) {
                        tile_entities.insert((x, y), entity);
                    }
                }
            }
        }

        World {
            map: Map {
                width: self.width,
                height: self.height,
                tiletypes: tiletypes,
                tiles: tiles,
            },
            entities: self.entities.iter().filter_map(|e| e.create(seed)).collect(),
            tile_entities: tile_entities,
            spawnables: Vec::new(),
        }
    }
}

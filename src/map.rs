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
use std::path::Path;

use vecmath::*;

/// This represents the logic for a type of entity.
pub trait EntityLogic {
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64, world: &mut WorldView) -> bool;
}

/// This represents the physical attributes of an entity.
pub struct EntityPhysics {
    pub pos: Vector2,
}

/// This is an entity in the world, with a position and pointer to the logic.
pub struct Entity {
    pub physics: EntityPhysics,
    pub logic: Box<EntityLogic>,
}

impl Entity {
    pub fn new<T: EntityLogic + 'static>(pos: Vector2, logic: T) -> Entity {
        Entity {
            physics: EntityPhysics {
                pos: pos,
            },
            logic: Box::new(logic),
        }
    }
}

type TileEntityFactory = &'static Fn(Tile, &TileType, (usize, usize)) -> Option<Entity>;

/// Definition of a tile type, referenced by tiles.
#[derive(Clone)]
struct TileType {
    /// Color to use to draw that tile.
    // TODO replace with texture
    color: [f32; 4],
    /// Damage suffered from touching that tile.
    damage: f32,
    /// Whether entities will collide with that tile, or pass through.
    collide: bool,
    /// Factory function (creates entity).
    tile_entity: Option<Box<TileEntityFactory>>,
}

/// A tile in the map, just references a TileType.
pub type Tile = u16;

pub trait Spawnable {
    fn spawn(&mut self, pos: &Vector2) -> (bool, Option<Entity>);
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
            },
            logic: logic,
        })
    }
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
}

/// Initial map definition, loaded from disk.
///
/// This can be turned into a live Map using `create()`.
pub struct MapFactory {
    width: usize,
    height: usize,
    nb_players: usize,
    tiletypes: Vec<TileType>,
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
        MapFactory {
            width: 100,
            height: 100,
            nb_players: 4,
            tiletypes: vec![
                TileType {
                    color: [0.5, 0.5, 0.5, 1.0],
                    damage: 0.0,
                    collide: true,
                    tile_entity: None,
                },
                TileType {
                    color: [0.0, 0.0, 0.0, 1.0],
                    damage: 0.0,
                    collide: false,
                    tile_entity: None,
                },
                TileType {
                    color: [0.0, 0.0, 1.0, 1.0],
                    damage: 0.0,
                    collide: false,
                    tile_entity: None,
                },
                TileType {
                    color: [1.0, 0.0, 0.0, 1.0],
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
    pub fn create(&self, seed: u32) -> World {
        let mut tile_entities = HashMap::new();

        let tiles = self.tiles.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = tiles[y * self.width + x];
                let tiletype = &self.tiletypes[tile as usize];
                if let Some(ref factory) = tiletype.tile_entity {
                    if let Some(entity) = factory(tile, tiletype, (x, y)) {
                        tile_entities.insert((x, y), entity);
                    }
                }
            }
        }

        World {
            map: Map {
                width: self.width,
                height: self.height,
                tiletypes: self.tiletypes.clone(),
                tiles: tiles,
            },
            entities: self.entities.iter().filter_map(|e| e.create(seed)).collect(),
            tile_entities: tile_entities,
            spawnables: Vec::new(),
        }
    }
}

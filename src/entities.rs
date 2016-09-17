use std::mem::swap;

use world::{EntityLogic, EntityPhysics, WorldView};

#[derive(Debug)]
pub struct Spawn {
}

impl Spawn {
    pub fn new() -> Spawn {
        Spawn {}
    }
}

impl EntityLogic for Spawn {
    fn update(&mut self, entity: &mut EntityPhysics, dt: f64, world: &mut WorldView) -> bool {
        // Move out all spawnables to avoid multiple borrows from Game
        let mut spawnables = Vec::new();
        swap(&mut spawnables, &mut world.spawnables);
        // Loop on spawnables, spawning at most one entity
        let mut spawned_one = false;
        let mut spawnables = spawnables.into_iter().filter_map(|mut spawnable| {
            if spawned_one {
                Some(spawnable)
            } else {
                let (keep, spawned) = spawnable.spawn(&entity.pos);
                if let Some(new_entity) = spawned {
                    world.entities.push(new_entity);
                    spawned_one = true;
                }
                if keep {
                    Some(spawnable)
                } else {
                    None
                }
            }
        }).collect::<Vec<_>>();
        // Put back the remaining spawnables
        swap(&mut spawnables, &mut world.spawnables);

        true
    }
}

use map::{Entity, EntityLogic, Spawnable};
use game::Game;

pub struct Spawn {
}

impl Spawn {
    pub fn new() -> Spawn {
        Spawn {}
    }
}

impl EntityLogic for Spawn {
    fn update(&mut self, entity: &mut Entity, dt: f64, game: &mut Game) -> bool {
        // Drain all spawnables to avoid multiple borrows from Game
        let spawnables:Vec<Box<Spawnable>> = game.map.spawnables.drain(..).collect();
        // Loop on spawnables, spawning at most one entity
        let mut spawned_one = false;
        let spawnables = spawnables.into_iter().filter_map(|mut spawnable| {
            if spawned_one {
                Some(spawnable)
            } else {
                let (keep, spawned) = spawnable.spawn(game);
                if let Some(entity) = spawned {
                    game.map.entities.push(entity);
                    spawned_one = true;
                }
                if keep {
                    Some(spawnable)
                } else {
                    None
                }
            }
        }).collect::<Vec<_>>();
        game.map.spawnables.extend(spawnables.into_iter());

        true
    }
}

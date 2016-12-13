use rand::{Rng, thread_rng};

use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use ::Window;
use ::sprites::{Sprite, SpriteManager};

struct Sequence {
    sprites: Vec<Sprite>,
    interval: f64,
}

impl Sequence {
    fn get_sprite(&self, time: f64) -> Option<Sprite> {
        self.sprites.get((time / self.interval) as usize).map(|s| s.clone())
    }

    fn duration(&self) -> f64 {
        self.sprites.len() as f64 * self.interval
    }
}

struct AnimationManager {
    character: CharacterMap,
}

impl AnimationManager {
    pub fn new(sprites: &SpriteManager, window: &mut Window,
        ) -> AnimationManager
    {
        let character = sprites.load(window, "character.png");
        AnimationManager {
            character: CharacterMap::new(sprites, window),
        }
    }
}

/// Plays through sequences and the state machine, using entity behavior from type parameters.
struct Animation<S: Clone + Debug, AM: AnimationMap<State=S>> {
    animation_map: AM,
    states: Vec<S>,
    sequence_time: f64,
}

impl<S: Clone + Debug, AM: AnimationMap<State=S>> Animation<S, AM> {
    fn new(animation_map: AM, initial_state: S) -> Animation<S, AM> {
        let states = animation_map.initial_states();
        Animation {
            animation_map: animation_map,
            states: states,
            sequence_time: 0.0,
        }
    }

    fn goto_state(&mut self, state: S) {
        let current = self.states[0].clone();
        self.states = self.animation_map.transition_to(current, Some(state));
        assert!(!self.states.is_empty());
        self.states.reverse();
        self.sequence_time = 0.0;
    }

    fn update(&mut self, dt: f64) {
        let mut current_state = self.states.last().expect("Animation has empty states vec").clone();
        let mut current_sequence = self.animation_map.sequence(current_state);
        self.sequence_time += dt;
        while self.sequence_time >= current_sequence.duration() {
            self.sequence_time -= current_sequence.duration();
            let last = self.states.pop().unwrap();
            if self.states.is_empty() {
                self.states = self.animation_map.transition_to(last, None);
                assert!(!self.states.is_empty());
                self.states.reverse();
            }
            current_state = self.states.last().unwrap().clone();
            current_sequence = self.animation_map.sequence(current_state);
        }
    }

    fn get_sprite(&self) -> Sprite {
        let state = self.states.last().expect("Animation has empty states vec");
        let sequence = self.animation_map.sequence(state.clone());
        sequence.get_sprite(self.sequence_time).expect("Animation exceeded sequence length")
    }
}

/// The part of an entity's logic that defines the transition state machine.
trait AnimationMap: Sized {
    type State;

    /// Initial states to go through.
    fn initial_states(&self) -> Vec<Self::State>;

    /// How to transition to the requested state from the current state.
    fn transition_to(&self, current: Self::State, other: Option<Self::State>,
        ) -> Vec<Self::State>;

    /// Get the sequence for the current state.
    fn sequence(&self, state: Self::State) -> Rc<Sequence>;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum CharacterState {
    IDLE_L,
    IDLE_R,
    IDLE_ANIM_1,
    IDLE_ANIM_2,
    RUNNING_L,
    RUNNING_R,
    SHOOTING_L,
    SHOOTING_R,
    REVERSING_LR,
    REVERSING_RL,
}

struct CharacterMap {
    sequences: HashMap<CharacterState, Sequence>,
}

impl CharacterMap {
    fn new(sprites: &SpriteManager, window: &mut Window) -> CharacterMap {
        let mut sequences = HashMap::new();
        unimplemented!();
        CharacterMap {
            sequences: sequences,
        }
    }
}

impl AnimationMap for CharacterMap {
    type State = CharacterState;

    fn initial_states(&self) -> Vec<CharacterState> {
        let s = CharacterState::IDLE_R;
        vec![
            s,
            s,
            s,
            s,
            s,
            *thread_rng().choose(
                &[CharacterState::IDLE_ANIM_1,
                    CharacterState::IDLE_ANIM_2]).unwrap(),
        ]
    }

    fn transition_to(&self, current: CharacterState, other: Option<CharacterState>,
    ) -> Vec<CharacterState>
    {
        // After being idle a couple times, do idle animation
        fn idle(right: bool) -> Vec<CharacterState> {
            let s = if right { CharacterState::IDLE_R } else { CharacterState::IDLE_L };
            vec![
            s,
            s,
            s,
            s,
            s,
            *thread_rng().choose(
                &[CharacterState::IDLE_ANIM_1,
                    CharacterState::IDLE_ANIM_2]).unwrap(),
            s,
            ]
        }

        match (current, other) {
            // When changing running direction, do reversing animation
            (CharacterState::RUNNING_L, Some(CharacterState::RUNNING_R)) => {
                vec![CharacterState::REVERSING_LR,
                CharacterState::RUNNING_R]
            }

            // Idle state
            (_, Some(s @ CharacterState::IDLE_L)) |
            (s @ CharacterState::IDLE_L, None) => idle(false),
            (_, Some(s @ CharacterState::IDLE_R)) |
            (s @ CharacterState::IDLE_R, None) => idle(true),

            // Go idle after shooting
            (CharacterState::SHOOTING_L, None) => idle(false),
            (CharacterState::SHOOTING_R, None) => idle(true),

            // Running simply loops
            (s @ CharacterState::RUNNING_L, None) |
            (s @ CharacterState::RUNNING_R, None) => vec![s],

            // Default cases
            (_, Some(s)) => vec![s],
            (s, None) => {
                debug!("Reached end of animation {:?} and don't know what to do", s);
                vec![CharacterState::IDLE_R]
            }
        }
    }

    fn sequence(&self, state: CharacterState) -> Rc<Sequence> {
        match state {
            _ => unimplemented!(),
        }
    }
}

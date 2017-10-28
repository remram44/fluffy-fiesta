use rand::{Rng, thread_rng};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use ::Window;
use ::sprites::{Sprite, SpriteManager};

trait Sequence: Debug {
    type Frame: Debug;

    fn interval(&self) -> f64;
    fn duration(&self) -> f64;
    fn get(&self, time: f64) -> Option<Self::Frame>;
}

#[derive(Debug)]
struct SpriteSequence {
    sprites: Vec<Sprite>,
    interval: f64,
}

impl Sequence for SpriteSequence {
    type Frame = Sprite;

    fn interval(&self) -> f64 {
        self.interval
    }

    fn duration(&self) -> f64 {
        self.sprites.len() as f64 * self.interval
    }

    fn get(&self, time: f64) -> Option<Sprite> {
        self.sprites.get((time / self.interval) as usize).map(|s| s.clone())
    }
}

/// The part of an entity's logic that defines the transition state machine.
trait AnimationMap: Sized {
    type Sequence: Sequence;
    type State: Copy;

    /// Initial states to go through.
    fn initial_states(&self) -> Cow<'static, [Self::State]>;

    /// How to transition to the requested state from the current state.
    fn transition_to(&self, current: Self::State, other: Option<Self::State>,
        ) -> Cow<'static, [Self::State]>;

    /// Get the sequence for the current state.
    fn sequence(&self, state: Self::State) -> Rc<Self::Sequence>;
}

/// Plays through sequences and the state machine, using given map.
struct Animation<S: 'static + Copy + Debug, AM: AnimationMap<State=S>> {
    animation_map: AM,
    states: Cow<'static, [S]>,
    state_idx: usize,
    sequence_time: f64,
}

impl<S: Copy + Debug, AM: AnimationMap<State=S>> Animation<S, AM> {
    fn new(animation_map: AM) -> Animation<S, AM> {
        let states = animation_map.initial_states();
        Animation {
            animation_map: animation_map,
            states: states,
            state_idx: 0,
            sequence_time: 0.0,
        }
    }

    fn goto_state(&mut self, state: S) {
        debug!("Animation::goto_state : state={:?}", state);
        let current = self.states[0];
        self.states = self.animation_map.transition_to(current, Some(state));
        assert!(!self.states.is_empty());
        self.state_idx = 0;
        self.sequence_time = 0.0;
        debug!("goto_state done, states={:?}", self.states);
    }

    fn update(&mut self, dt: f64) {
        let mut current_state = self.states.get(self.state_idx).expect("Animation has empty states vec").clone();
        let mut current_sequence = self.animation_map.sequence(current_state);
        self.sequence_time += dt;
        debug!("Animation::update : dt={:?}, sequence_time={:?}",
               dt, self.sequence_time);
        while self.sequence_time >= current_sequence.duration() {
            debug!("sequence_time={:?}, moving to next state #{:?}",
                   self.sequence_time, self.state_idx + 1);
            self.sequence_time -= current_sequence.duration();
            let last = current_state;
            self.state_idx += 1;
            if self.state_idx >= self.states.len() {
                self.states = self.animation_map.transition_to(last, None);
                assert!(!self.states.is_empty());
                self.state_idx = 0;
                debug!("no more states, transitioning, now {:?} states",
                       self.states.len());
            }
            current_state = self.states[self.state_idx];
            debug!("state is now {:?}", current_state);
            current_sequence = self.animation_map.sequence(current_state);
        }
        debug!("update done, states={:?}, idx={:?}, time={:?}",
               self.states, self.state_idx, self.sequence_time);
    }

    fn get(&self) -> <<AM as AnimationMap>::Sequence as Sequence>::Frame {
        let state = self.states.get(self.state_idx).expect("Animation has empty states vec").clone();
        let sequence = self.animation_map.sequence(state);
        let frame = sequence.get(self.sequence_time).expect("Animation exceeded sequence length");
        debug!("Animation::get : state={:?}, sequence={:?}, time={:?}, frame={:?}",
               state, sequence, self.sequence_time, frame);
        frame
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum CharacterState {
    IdleLeft,
    IdleRight,
    IdleAnim1,
    IdleAnim2,
    RunningLeft,
    RunningRight,
    ShootingLeft,
    ShootingRight,
    ReversingLeftToRight,
    ReversingRightToLeft,
}

struct CharacterMap {
    sequences: HashMap<CharacterState, SpriteSequence>,
}

impl CharacterMap {
    fn new(sprites: &SpriteManager, window: &mut Window) -> CharacterMap {
        let mut sequences = HashMap::new();
        unimplemented!();
        CharacterMap {
            sequences: sequences,
        }
    }

    fn idle(right: bool) -> Cow<'static, [CharacterState]> {
        use self::CharacterState::*;
        // After being idle a couple times, do idle animation
        macro_rules! seq {
            ($st:expr, $anim:expr) => (
                Cow::Borrowed(&[$st, $st, $st, $st, $st, $anim])
            )
        }
        if right {
            if thread_rng().gen::<bool>() {
                seq!(IdleRight, IdleAnim1)
            } else {
                seq!(IdleRight, IdleAnim2)
            }
        } else {
            if thread_rng().gen::<bool>() {
                seq!(IdleLeft, IdleAnim1)
            } else {
                seq!(IdleLeft, IdleAnim2)
            }
        }
    }
}

impl AnimationMap for CharacterMap {
    type Sequence = SpriteSequence;
    type State = CharacterState;

    fn initial_states(&self) -> Cow<'static, [CharacterState]> {
        Self::idle(true)
    }

    fn transition_to(&self, current: CharacterState, other: Option<CharacterState>,
    ) -> Cow<'static, [CharacterState]>
    {
        match (current, other) {
            // When changing running direction, do reversing animation
            (CharacterState::RunningLeft, Some(CharacterState::RunningRight)) => {
                Cow::Borrowed(&[CharacterState::ReversingLeftToRight,
                                CharacterState::RunningRight])
            }

            // Idle state
            (_, Some(CharacterState::IdleLeft)) |
            (CharacterState::IdleLeft, None) => Self::idle(false),
            (_, Some(CharacterState::IdleRight)) |
            (CharacterState::IdleRight, None) => Self::idle(true),

            // Go idle after shooting
            (CharacterState::ShootingLeft, None) => Self::idle(false),
            (CharacterState::ShootingRight, None) => Self::idle(true),

            // Running simply loops
            (CharacterState::RunningLeft, None) => Cow::Borrowed(&[CharacterState::RunningLeft]),
            (CharacterState::RunningRight, None) => Cow::Borrowed(&[CharacterState::RunningRight]),

            // Default cases
            (_, Some(s)) => Cow::Owned(vec![s]),
            (s, None) => {
                debug!("Reached end of animation {:?} and don't know what to do", s);
                Self::idle(true)
            }
        }
    }

    fn sequence(&self, state: CharacterState) -> Rc<SpriteSequence> {
        match state {
            _ => unimplemented!(),
        }
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

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::rc::Rc;

    use super::{Animation, AnimationMap, Sequence};

    #[derive(Debug)]
    struct TestSequence {
        id: usize,
    }

    impl Sequence for TestSequence {
        type Frame = usize;

        fn interval(&self) -> f64 {
            1.0
        }

        fn duration(&self) -> f64 {
            5.0
        }

        fn get(&self, time: f64) -> Option<usize> {
            Some(self.id + time as usize)
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum TestState {
        Idle = 10,
        IdleAnim = 20,
        Running = 30,
        Stopping = 40,
    }

    struct TestMap {
    }

    impl AnimationMap for TestMap {
        type Sequence = TestSequence;
        type State = TestState;

        fn initial_states(&self) -> Cow<'static, [TestState]> {
            use self::TestState::*;
            Cow::Borrowed(&[
                Idle,
                Idle,
                IdleAnim,
            ])
        }

        fn transition_to(&self, current: TestState, other: Option<TestState>
        ) -> Cow<'static, [TestState]>
        {
            use self::TestState::*;
            match (current, other) {
                (Running, Some(Idle)) => Cow::Borrowed(&[Stopping, Idle]),
                (_, Some(t)) => Cow::Owned(vec![t]),
                (Running, None) => Cow::Borrowed(&[Running]),
                (_, None) => Cow::Borrowed(&[Idle, Idle, IdleAnim]),
            }
        }

        fn sequence(&self, state: TestState) -> Rc<TestSequence> {
            debug!("TestMap::sequence : state={:?}", state);
            Rc::new(TestSequence { id: state.clone() as usize })
        }
    }

    #[test]
    fn test_animation() {
        let mut animation = Animation::new(TestMap {});
        // starts with states=[Idle, Idle, IdleAnim], idx=0
        assert_eq!(animation.get(), 10);
        animation.update(9.2);
        // moves to idx=1
        assert_eq!(animation.get(), 14);
        animation.update(1.0);
        // moves to idx=2
        assert_eq!(animation.get(), 20);
        animation.update(8.0);
        // moves to idx=3
        // runs out of states, call transition_to(., None)
        // states=[Idle, Idle, IdleAnim], idx=0
        assert_eq!(animation.get(), 13);
        animation.update(3.0);
        // moves to idx=1
        assert_eq!(animation.get(), 11);
        animation.goto_state(TestState::Running);
        // states=[Running], idx=0
        assert_eq!(animation.get(), 30);
        animation.update(8.2);
        assert_eq!(animation.get(), 33);
        animation.goto_state(TestState::Idle);
        assert_eq!(animation.get(), 40);
        animation.update(3.2);
        assert_eq!(animation.get(), 43);
        animation.update(3.0);
        assert_eq!(animation.get(), 11);
    }
}

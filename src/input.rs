use piston;
use piston::input::*;

const INPUT_THRESHOLD: f32 = 0.8;

pub struct PlayerInput {
    d_right: bool,
    d_left: bool,
    d_up: bool,
    d_down: bool,
    d_jump: bool,

    a_x: f32,
    a_y: f32,
}

impl PlayerInput {
    fn new() -> PlayerInput {
        PlayerInput {
            d_right: false,
            d_left: false,
            d_up: false,
            d_down: false,
            d_jump: false,
            a_x: 0.0,
            a_y: 0.0,
        }
    }

    fn da_input(a: f32, d_neg: bool, d_pos: bool) -> f32 {
        let value = a +
            (if d_neg { -1.0 } else { 0.0 }) +
            (if d_pos { 1.0 } else { 0.0 });
        if value < -1.0 {
            -1.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        }
    }

    pub fn x(&self) -> f32 {
        PlayerInput::da_input(self.a_x, self.d_left, self.d_right)
    }

    pub fn y(&self) -> f32 {
        PlayerInput::da_input(self.a_y, self.d_down, self.d_up)
    }

    pub fn jump(&self) -> bool {
        self.d_up || self.a_y > INPUT_THRESHOLD
    }
}

/// The input manager, mapping input events into player actions.
pub struct InputManager {
    players: Vec<PlayerInput>,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            players: vec![PlayerInput::new()],
        }
    }

    pub fn handle_event(&mut self, event: &piston::input::Event<piston::input::Input>) {
        if let Some((key, pressed)) = if let Some(Button::Keyboard(key)) = event.press_args() {
            info!("Pressed key '{:?}'", key);
            Some((key, true))
        } else if let Some(Button::Keyboard(key)) = event.release_args() {
            info!("Released key '{:?}", key);
            Some((key, false))
        } else {
            None
        } {
            if key == Key::Right {
                self.players[0].d_right = pressed;
            } else if key == Key::Left {
                self.players[0].d_left = pressed;
            } else if key == Key::Up {
                self.players[0].d_up = pressed;
            } else if key == Key::Down {
                self.players[0].d_down = pressed;
            }
        }
    }

    pub fn update(&mut self, dt: f64) {
    }

    pub fn player_input(&self, player: usize) -> Option<&PlayerInput> {
        self.players.get(player)
    }
}

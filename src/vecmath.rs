use vecmath_lib;

pub use vecmath_lib::{vec2_add, vec2_sub, vec2_mul, vec2_scale};

pub type Vector2 = vecmath_lib::Vector2<f32>;

pub trait Vector2Ext {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

impl Vector2Ext for Vector2 {
    fn x(&self) -> f32 {
        self[0]
    }

    fn y(&self) -> f32 {
        self[1]
    }
}

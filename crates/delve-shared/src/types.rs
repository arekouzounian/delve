use crate::math::Vec3;

#[allow(unused)]
pub struct Collision {
    pub collision_position: Vec3,
    pub surface_normal: Vec3,
    pub length_coefficient: f32,
}

impl Collision {
    pub fn closest(a: Self, b: Self) -> Self {
        if a.length_coefficient < b.length_coefficient {
            return a;
        }

        b
    }
}

// All the fields needed for an Entity implementation.
#[derive(Default, Debug)]
pub struct EntityFields {
    pub rotation: crate::math::Mat3,
    pub acceleration: Vec<(crate::math::Vec3, f32)>,
    pub velocity: Vec3,
    pub position: Vec3,
    pub gravity_multiplier: f32,
}

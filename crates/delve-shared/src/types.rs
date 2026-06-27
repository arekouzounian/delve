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
// 06-27-26: yeah im looking at this way later and this is pretty contrived and not performant.
// I might just overhaul this with something similar to a stripped down ecs. this'll do for now tho
#[derive(Default, Debug)]
pub struct EntityFields {
    pub rotation: crate::math::Mat3,
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub position: Vec3,
    pub invisible: bool,
}

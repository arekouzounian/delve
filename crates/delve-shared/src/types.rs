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

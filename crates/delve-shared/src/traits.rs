use crate::math::{Mat3, Vec3};
use crate::types::Collision;

pub trait IntersectableEntity: Entity + RayIntersect {}
impl<T: Entity + RayIntersect> IntersectableEntity for T {}

pub trait Entity {
    fn rotation(&self) -> &Mat3;
    fn rotation_mut(&mut self) -> &mut Mat3;

    fn position(&self) -> &Vec3;
    fn position_mut(&mut self) -> &mut Vec3;

    fn velocity(&self) -> &Vec3;
    fn velocity_mut(&mut self) -> &mut Vec3;

    fn acceleration(&self) -> &Vec3;
    fn acceleration_mut(&mut self) -> &mut Vec3;

    /// applies decay to acceleration, acceleration to velocity,
    /// then velocity to position.
    fn apply_forces(&mut self);
}

pub trait RayIntersect {
    /// Checks if there is a collision for this ray on this entity
    fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision>;
}

pub trait Collide: Entity {
    fn is_colliding(&self, ray_collision: Collision) -> bool;
}

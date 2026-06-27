use crate::math::Vec3;
use crate::types::{Collision, EntityFields};

pub trait IntersectableEntity: Entity + RayIntersect {}
impl<T: Entity + RayIntersect> IntersectableEntity for T {}

pub trait Entity {
    fn entity_fields(&self) -> &EntityFields;
    fn entity_fields_mut(&mut self) -> &mut EntityFields;
    /// applies decay to acceleration, acceleration to velocity,
    /// then velocity to position.
    fn apply_forces(&mut self);
}

pub trait RayIntersect {
    /// Checks if there is a collision for this ray on this entity
    fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision>;

    /// finds the closest point on the surface to the origin_point
    fn closest_point(&self, origin_point: Vec3) -> Vec3;
}

pub trait Collide: Entity {
    fn is_colliding(&self, ray_collision: Collision) -> bool;
}

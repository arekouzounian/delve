use crate::math::{Mat3, Vec3};
use crate::types::Collision;

pub trait Entity {
    /// Sets the gravity multiplier for the entity.
    /// <0.0 => negative gravity
    /// 0.0  => zero gravity
    /// 1.0  => normal gravity
    /// >1.0 => increased gravity
    fn set_gravity(&mut self, multiplier: f32);

    /// Sets the rotation matrix for the entity
    fn set_rotation(&mut self, rotation_matrix: Mat3);

    /// Set the position vector for the entity
    fn set_position(&mut self, position: Vec3);

    /// Set the velocity vector for the entity.
    /// This will in turn be applied to the position vector
    fn set_velocity(&mut self, velocity: Vec3);

    /// Add to the acceleration vector for this entity.
    /// This will in turn be applied to the velocity vector.
    /// The decay factor decays this acceleration vector on each application;
    /// close to 0.0 => decays very quickly
    /// close to 1.0 => decays very slowly
    fn add_acceleration(&mut self, acceleration: Vec3, decay: f32);

    /// clears all acceleration vectors
    fn clear_acceleration(&mut self);

    /// applies decay to acceleration, acceleration to velocity,
    /// then velocity to position.
    fn apply_forces(&mut self);
}

pub trait RayIntersect {
    /// Checks if there is a collision for this ray on this entity
    fn intersect(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<Collision>;
}

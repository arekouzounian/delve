use crate::types::EntityFields;

pub trait Entity {
    fn entity_fields(&self) -> &EntityFields;
    fn entity_fields_mut(&mut self) -> &mut EntityFields;

    /// applies decay to acceleration, acceleration to velocity,
    /// then velocity to position.
    fn apply_forces(&mut self);
}

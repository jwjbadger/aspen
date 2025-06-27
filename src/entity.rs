/// An entity in the ECS architecture
///
/// A newtype around a [`u32`] that represents the id for an entity. This id is used to index
/// components in order to associate data with entities.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Entity(u32);

impl Entity {
    pub(crate) fn new(id: u32) -> Self {
        Entity(id)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Entity(u32);

impl Entity {
    pub fn new(id: u32) -> Self {
        Entity(id)
    }
}

pub struct EntityBuilder {}

impl EntityBuilder {}

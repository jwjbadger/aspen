use crate::Entity;
use std::any::TypeId;
use std::collections::HashMap;

/// A wrapper for data that associates it with an entity.
///
/// Currently has no use in the publically accessible API although it may eventually gain some use.
#[derive(Clone, Debug)]
pub struct Component<T: 'static> {
    pub(crate) data: HashMap<Entity, T>,
    pub(crate) type_id: std::any::TypeId,
}

impl<T> Component<T> {
    pub(crate) fn new(type_id: TypeId) -> Self {
        Component {
            data: HashMap::new(),
            type_id,
        }
    }

    pub(crate) fn entities(&self) -> Vec<Entity> {
        self.data.keys().cloned().collect()
    }

    pub(crate) fn add_entity(&mut self, entity: Entity, component: T) {
        self.data.insert(entity, component);
    }

    pub(crate) fn remove_entity(&mut self, entity: &Entity) {
        self.data.remove(&entity);
    }
}

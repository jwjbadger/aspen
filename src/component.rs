use crate::Entity;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Component<T: 'static> {
    pub data: HashMap<Entity, T>,
    pub type_id: std::any::TypeId,
}

impl<T> Component<T> {
    pub fn new(type_id: TypeId) -> Self {
        Component {
            data: HashMap::new(),
            type_id,
        }
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.data.keys().cloned().collect()
    }

    pub fn add_entity(&mut self, entity: Entity, component: T) {
        self.data.insert(entity, component);
    }

    pub fn remove_entity(&mut self, entity: &Entity) {
        self.data.remove(&entity);
    }
}

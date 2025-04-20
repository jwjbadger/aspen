use crate::{component::Component, entity::Entity};
use std::any::Any;
use std::any::TypeId;
use std::collections::HashSet;
use std::rc::Rc;

// Somehow has to match typeid to component
pub struct Query<'a> {
    matches: Vec<&'a mut Component<Rc<dyn Any>>>,
}

impl<'a> Query<'a> {
    pub fn new(haystack: &'a mut Vec<Component<Rc<dyn Any>>>, filter: &HashSet<TypeId>) -> Self {
        Self {
            matches: haystack
                .iter_mut()
                .filter(|e| filter.contains(&(*e).type_id))
                .collect(),
        }
    }

    pub fn get<T: 'static>(&mut self) -> Vec<Component<Rc<T>>> {
        self.matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .map(|e| {
                let data = e
                    .data
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, v.downcast::<T>().unwrap()))
                    .collect::<std::collections::HashMap<Entity, Rc<T>>>();
                Component::<Rc<T>> {
                    data,
                    type_id: e.type_id,
                }
            })
            .collect()
    }

    pub fn set<T: 'static + Clone>(&mut self, entity: Entity, new: T) {
        self.matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .for_each(|e| {
                e.data.iter_mut().for_each(|(k, mut v)| {
                    if *k == entity {
                        *Rc::get_mut(&mut v).unwrap().downcast_mut::<T>().unwrap() = new.clone();
                    }
                });
            });
    }
}

pub trait SystemInterface {
    fn execute(&mut self, query: Query);
    fn components(&self) -> &HashSet<TypeId>;
}

pub struct System {
    pub components: HashSet<TypeId>,
    pub executable: fn(Query),
}

impl System {
    pub fn new(components: Vec<TypeId>, executable: fn(Query)) -> Self {
        Self {
            components: components.into_iter().collect(),
            executable,
        }
    }
}

impl SystemInterface for System {
    fn execute(&mut self, query: Query) {
        (self.executable)(query)
    }

    fn components(&self) -> &HashSet<TypeId> {
        &self.components
    }
}

pub struct ResourcedSystem<T> {
    pub components: HashSet<TypeId>,
    pub executable: fn(Query, &T),
    pub resource: T,
}

impl<T> ResourcedSystem<T> {
    pub fn new(components: Vec<TypeId>, resource: T, executable: fn(Query, &T)) -> Self {
        Self {
            components: components.into_iter().collect(),
            resource,
            executable,
        }
    }
}

impl<T> SystemInterface for ResourcedSystem<T> {
    fn execute(&mut self, query: Query) {
        (self.executable)(query, &self.resource)
    }

    fn components(&self) -> &HashSet<TypeId> {
        &self.components
    }
}

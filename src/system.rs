use crate::{component::Component, entity::Entity};
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

// Somehow has to match typeid to component
pub struct Query<'a> {
    matches: Vec<&'a mut Component<Arc<Mutex<dyn Any>>>>,
}

impl<'a> Query<'a> {
    pub fn new(
        haystack: &'a mut Vec<Component<Arc<Mutex<dyn Any>>>>,
        filter: &HashSet<TypeId>,
    ) -> Self {
        Self {
            matches: haystack
                .iter_mut()
                .filter(|e| filter.contains(&(*e).type_id))
                .collect(),
        }
    }

    pub fn each<T: 'static>(&mut self, f: fn(&mut T)) {
        self.matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .for_each(|e| {
                e.data.iter_mut().for_each(|(_, v)| {
                    f(v.lock().unwrap().downcast_mut::<T>().unwrap());
                });
            });
    }

    pub fn all<T: 'static>(&mut self, f: fn(HashMap<Entity, &mut T>)) {
        let mut data = self
            .matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .flat_map(|e| {
                e.data
                    .iter()
                    .map(|(k, v)| (k.clone(), v.lock().unwrap())) // TODO: can we directly deref
                    .collect::<Vec<(Entity, MutexGuard<dyn Any>)>>()
            })
            .collect::<HashMap<Entity, MutexGuard<dyn Any>>>();

        let matches = data
            .iter_mut()
            .map(|(k, v)| (k.clone(), v.downcast_mut::<T>().unwrap()))
            .collect::<HashMap<Entity, &mut T>>();

        f(matches);
    }

    /*pub fn get<T: 'static>(&mut self) -> Vec<Component<Arc<Mutex<T>>>> {
        self.matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .map(|e| {
                let data = e
                    .data
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, v.downcast::<T>().unwrap()))
                    .collect::<std::collections::HashMap<Entity, Arc<Mutex<T>>>>();
                Component::<Arc<Mutex<T>>> {
                    data,
                    type_id: e.type_id,
                }
            })
            .collect()
    }*/

    /*pub fn set<T: 'static + Clone>(&mut self, entity: Entity, new: T) {
        self.matches
            .iter_mut()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .for_each(|e| {
                e.data.iter_mut().for_each(|(k, mut v)| {
                    if *k == entity {
                        *(v.lock().unwrap().downcast_mut::<T>()) = new.clone();
                    }
                });
            });
    }*/
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

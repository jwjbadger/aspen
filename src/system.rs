use crate::{component::Component, entity::Entity};
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

// Somehow has to match typeid to component
#[derive(Debug)]
pub struct Query<'a> {
    matches: Vec<&'a mut Component<Arc<Mutex<dyn Any>>>>,
}

impl<'a> Query<'a> {
    pub fn new(
        haystack: &'a mut Vec<Component<Arc<Mutex<dyn Any>>>>,
        filter: &HashSet<TypeId>,
    ) -> Self {
        // TODO: unexpected behavior doesn't remove components that aren't in both
        Self {
            matches: haystack
                .iter_mut()
                .filter(|e| filter.contains(&(*e).type_id))
                .collect(),
        }
    }

    pub fn get_entities<T: 'static>(&self) -> Vec<Entity> {
        self.matches
            .iter()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .flat_map(|e| {
                e.data
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect::<Vec<Entity>>()
            })
            .collect()
    }

    // it is guaranteed that dyn Any is of type T, but it seems impossible to downcast the Mutex
    // without first turning it into a MutexGuard
    // TODO: fix this
    pub fn get<T: 'static>(&self, ent: &Entity) -> Option<Arc<Mutex<dyn Any>>> {
        // TODO: check to make sure there's only one of each type of component
        match self
            .matches
            .iter()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .next()
        {
            Some(component) => component.data.get(ent).map(|v| v.clone()),
            None => None,
        }
    }

    pub fn get_all<T: 'static>(&self) -> HashMap<Entity, Arc<Mutex<dyn Any>>> {
        self.matches
            .iter()
            .filter(|e| e.type_id == TypeId::of::<T>())
            .flat_map(|e| {
                e.data
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<(Entity, Arc<Mutex<dyn Any>>)>>()
            })
            .collect()
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

    pub fn all<T: 'static>(&mut self, f: impl FnOnce(HashMap<Entity, &mut T>)) {
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
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.downcast_mut::<T>()
                        .expect(&format!("couldn't downcast value for ent: {:?}", k)),
                )
            })
            .collect::<HashMap<Entity, &mut T>>();

        f(matches);
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

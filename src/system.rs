use crate::{component::Component, entity::Entity};
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

/// A query for entities containing specific components
///
/// Contains matches for a specific combination of components in order for the systems to be able
/// to modify data held by components. Cannot be instantiated by anything other than the world
/// although instances of it will be passed to systems.
#[derive(Debug)]
pub struct Query<'a> {
    matches: Vec<&'a mut Component<Arc<Mutex<dyn Any>>>>,
}

impl<'a> Query<'a> {
    pub(crate) fn new(
        haystack: &'a mut Vec<Component<Arc<Mutex<dyn Any>>>>,
        filter: &HashSet<TypeId>,
    ) -> Self {
        // TODO: unexpected behavior doesn't remove components that aren't in both
        // how has this not been an issue yet
        Self {
            matches: haystack
                .iter_mut()
                .filter(|e| filter.contains(&(*e).type_id))
                .collect(),
        }
    }

    /// Returns all entities that match the given query and are of a specific component
    ///
    /// This should be the same as getting all entities that match the given query, but because of
    /// faulty logic in the creation of queries, all entities that have any component in the query
    /// will currently match it, meaning this function will return different entities depending on
    /// the associated component.
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

    /// Returns a component on a particular entity if that component exists
    ///
    /// Returns the data without downcasting it so as to prevent the MutexGuard from being
    /// destroyed when returned from the function, which means the return value will have to be
    /// manually downcasted upon retrieval in order to operate on it.
    pub fn get<T: 'static>(&self, ent: &Entity) -> Option<Arc<Mutex<dyn Any>>> {
        // it is guaranteed that dyn Any is of type T, but it seems impossible to downcast the Mutex
        // without first turning it into a MutexGuard
        // TODO: fix this
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

    /// Returns all entities and components of a certain type
    ///
    /// The same restrictions apply as do on [`get`] where the result will be returned without
    /// being downcasted, requiring manual downcasting to get a [`MutexGuard`] of the desired type.
    ///
    /// [`get`]: Self::get()
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

    /// Applies a function on every entity of a specific component
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

    /// Applies a function to all entities of a specific component
    ///
    /// Better in most cases than [`get_all`] because the data doesn't have to be manually
    /// downcasted. In certain circumstances, this will be impossible to use due to current
    /// restrictions with only using a single component at a time (e.g. when combining data from
    /// two components, both cannot use this method because of borrowing issues).
    ///
    /// [`get_all`]: Self::get_all()
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

/// Defines the interface for all systems
///
/// Allows for various types of systems that can pull data in or store it in a variety of manners
/// in order to iteract with resources external to the entities.
pub trait SystemInterface {
    /// Called when the system runs.
    fn execute(&mut self, query: Query);
    /// Returns the [`TypeId`]s of all components upon which the system operates.
    fn components(&self) -> &HashSet<TypeId>;
}

/// The standard system.
///
/// Stores the types of components it operates on and runs a function that depends on nothing
/// external.
pub struct System {
    /// The components on which the system operates.
    pub components: HashSet<TypeId>,
    /// The function to execute when the system runs.
    pub executable: fn(Query),
}

impl System {
    /// Creates a new system based on the [`TypeId`]s of the components on which it operates and a
    /// function pointer that  will be executed when the system is.
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

/// Another basic system that pulls in an external resource.
///
/// Stores data upon the creation of the system that is passed into the function that runs when the
/// system executes. Used internally for systems that require access to the renderer to provide
/// them access without creating global state.
pub struct ResourcedSystem<T> {
    /// The components on which the system operates.
    pub components: HashSet<TypeId>,
    /// The function to execute when the system runs. Takes in the query and an immutable reference
    /// to the resource.
    pub executable: fn(Query, &T),
    /// The resource that should be accessible when the system runs.
    pub resource: T,
}

impl<T> ResourcedSystem<T> {
    /// Creates a new system based on the [`TypeId`]s of the components on which it operates, a
    /// function pointer that  will be executed when the system is, and the resource that should be
    /// stored by the system.
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

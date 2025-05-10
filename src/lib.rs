pub mod camera;
pub mod input;
pub mod component;
pub mod entity;
pub mod graphics;
pub mod mesh;
pub mod os;
pub mod system;

pub use crate::{
    component::Component,
    entity::Entity,
    graphics::{Renderer, WgpuRenderer},
    os::App,
    system::{Query, System, SystemInterface},
};

use std::{any::Any, rc::Rc, time::Instant};

pub struct World<'a> {
    entities: Vec<Entity>,
    components: Vec<Component<Rc<dyn Any>>>,
    fixed_systems: Vec<Box<dyn SystemInterface + 'a>>,
    dependent_systems: Vec<Box<dyn SystemInterface + 'a>>,
    current_id: u32,
    period: f32,
    pub previous_time: Instant,
    accumulator: f32,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> World<'a> {
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    pub fn tick(&mut self) {
        let current_time = Instant::now();
        let delta_time = self.previous_time.elapsed();
        self.previous_time = current_time;

        self.accumulator += delta_time.as_secs_f32();
        while self.accumulator >= self.period {
            self.fixed_systems
                .iter_mut()
                .for_each(|s| s.execute(Query::new(&mut self.components, s.components())));

            self.accumulator -= self.period;
        }

        self.dependent_systems
            .iter_mut()
            .for_each(|s| s.execute(Query::new(&mut self.components, s.components())));
    }

    pub fn new_entity(&mut self) -> Entity {
        self.entities.push(Entity::new(self.current_id));
        self.current_id += 1;
        *self.entities.last().unwrap()
    }

    pub fn add_component<T: Any + Clone>(&mut self, entity: Entity, data: T) {
        for e in self.components.iter_mut() {
            if e.type_id == std::any::TypeId::of::<T>() {
                e.add_entity(entity, Rc::new(data.clone()));
                return;
            }
        }

        self.components
            .push(Component::new(std::any::TypeId::of::<T>()));
        self.components
            .last_mut()
            .unwrap()
            .add_entity(entity, Rc::new(data.clone()));
    }

    pub fn add_fixed_system<T: SystemInterface + 'a>(&mut self, system: T) {
        self.fixed_systems.push(Box::new(system));
    }

    pub fn add_dependent_system<T: SystemInterface + 'a>(&mut self, system: T) {
        self.dependent_systems.push(Box::new(system));
    }
}

pub struct WorldBuilder {
    frequency: u16,
}

impl Default for WorldBuilder {
    fn default() -> Self {
        WorldBuilder { frequency: 60 }
    }
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder::default()
    }

    pub fn with_frequency(mut self, frequency: u16) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn build<'a>(self) -> World<'a> {
        World {
            entities: Vec::new(),
            components: Vec::new(),
            fixed_systems: Vec::new(),
            dependent_systems: Vec::new(),
            period: 1.0 / f32::from(self.frequency),
            current_id: 0,
            previous_time: Instant::now(),
            accumulator: 0.0,
            phantom: std::marker::PhantomData,
        }
    }
}

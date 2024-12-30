pub mod component;
pub mod entity;
pub mod event_loop;
pub mod system;

pub use event_loop::EventLoop;

use component::Component;
use entity::Entity;
use std::any::Any;
use std::rc::Rc;
use system::{Query, System};

pub struct World {
    event_loop: Option<EventLoop>,
    entities: Vec<Entity>,
    components: Vec<Component<Rc<dyn Any>>>,
    fixed_systems: Vec<System>,
    dependent_systems: Vec<System>,
    current_id: u32,
}

impl World {
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    pub fn run(&mut self) {
        if let Some(event_loop) = &mut self.event_loop {
            event_loop.begin(|fixed| {
                if fixed {
                    self.fixed_systems
                        .iter()
                        .for_each(|e| e.execute(Query::new(&mut self.components, &e.components)));
                } else {
                    self.dependent_systems.iter().for_each(|e| {
                        e.execute(Query::new(&mut self.components, &e.components));
                    });
                }
            });
        } else {
            loop {
                self.dependent_systems.iter().for_each(|e| {
                    e.execute(Query::new(&mut self.components, &e.components));
                });
            }
        }
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

    pub fn add_fixed_system(&mut self, system: System) {
        self.fixed_systems.push(system);
    }

    pub fn add_dependent_system(&mut self, system: System) {
        self.dependent_systems.push(system);
    }
}

#[derive(Default)]
pub struct WorldBuilder {
    fixed_loop_period: Option<u16>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        WorldBuilder::default()
    }

    pub fn timed_loop(mut self, period: u16) -> Self {
        self.fixed_loop_period = Some(period);
        self
    }

    pub fn build(&self) -> World {
        World {
            event_loop: self.fixed_loop_period.map(|e| EventLoop::new(e)),
            entities: Vec::new(),
            components: Vec::new(),
            fixed_systems: Vec::new(),
            dependent_systems: Vec::new(),
            current_id: 0,
        }
    }
}

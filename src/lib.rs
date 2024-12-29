mod component;
mod entity;
mod event_loop;
mod system;

pub use event_loop::EventLoop;

use component::Component;
use entity::Entity;
use system::{Query, System};

pub struct World {
    event_loop: Option<EventLoop>,
    entities: Vec<Entity>,
    components: Vec<Box<dyn Component>>,
    fixed_systems: Vec<System>,
    dependent_systems: Vec<System>
}

impl World {
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    pub fn run(&mut self) {
        let dependent = |alpha: f32| {};

        if let Some(event_loop) = &mut self.event_loop {
            event_loop.begin(|| {}, Some(dependent));
        } else {
            loop {
                dependent(0.0);
            }
        }
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
            dependent_systems: Vec::new()
        }
    }
}

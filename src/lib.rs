//! A simple (in progress) WGPU-based game engine built in Rust and focused on customization 
//! especially in the realm of simulation. It relates entities and components as archetypes.
//!
//! Focuses on three main goals:
//! - **Customizable**: Every task completed by the user should be customizable and all engine code
//! (with exception of the core ECS system) should be replaceable.
//! - **Simple**: Tasks should be simple to complete and intuitive to learn or well-documented when
//! unintuitive.
//! - **Data-driven**: Data should be the focus of all tasks as dictated by the ECS paradigm.
//!
//! See the [`README`] for more information.
//!
//! [`README`]: https://github.com/jwjbadger/aspen
#![warn(missing_docs)]

/// Handles everything related to the camera, which provides a point of access to the world,
/// allowing it to be rendered to the screen.
pub mod camera;
/// Handles everything related to WGPU textures, allowing them to be built and used by WGPU.
pub mod texture;
/// Handles the component side of ECS. Rarely used externally.
pub mod component;
/// Handles the entity side of ECS.
pub mod entity;
/// Primarily handles renderers and renderable objects.
pub mod graphics;
/// Handles all input.
pub mod input;
/// Holds all structures required to create something that is renderable.
pub mod mesh;
/// Used for GUI applications to handle operating system specific tasks (e.g. requesting input and
/// creating windows).
pub mod os;
/// Handles the system side of ECS.
pub mod system;

pub use crate::{
    component::Component,
    entity::Entity,
    graphics::{Renderer, WgpuRenderer},
    os::App,
    system::{Query, System, SystemInterface},
};

use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::Instant,
};

/// The world in which all entities, components, and system lie
///
/// Worlds cannot be created directly but rather with the [`WorldBuilder`] struct. They handle all
/// entities along with operating dependent and fixed systems every time the simulation ticks.
///
/// The world is used equally for visual and non-visual simulations---the only thing that changes is
/// how the world is stored and how certain special systems are handled. See [`App`] for more
/// information on GUI applications.
///
/// [`WorldBuilder`]: crate::WorldBuilder
/// [`App`]: crate::os::App
pub struct World<'a> {
    entities: Vec<Entity>,
    components: Vec<Component<Arc<Mutex<dyn Any>>>>,
    fixed_systems: Vec<Box<dyn SystemInterface + 'a>>,
    dependent_systems: Vec<Box<dyn SystemInterface + 'a>>,
    current_id: u32,
    period: f32,
    previous_time: Instant,
    accumulator: f32,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> World<'a> {
    /// Returns a new [`WorldBuilder`], which is used to create the world.
    pub fn builder() -> WorldBuilder {
        WorldBuilder::new()
    }

    /// Creates a new world that is ready for immediate usage.
    ///
    /// Should be used for all use cases where the world will be used immediately after creation
    /// because the timestep used for the first tick will be dependent on the difference in time
    /// between the creation of the world and the first tick.
    pub fn new(frequency: u16) -> Self {
        WorldBuilder::new().with_frequency(frequency).build()
    }

    /// Represents a time step within the simulation that should be called within a loop for non-GUI
    /// applications.
    ///
    /// During each time step, two primary tasks are executed:
    /// - All fixed systems are ran as many times as they need to be in order to make up the time
    /// between ticks. For instance, if fixed systems are intended to be ran ten times per second
    /// and 1.25 seconds have passed, the systems will be ran twelve times, with the thirteenth
    /// occurring the next time tick is called and at least 0.05 seconds have passed.
    /// - All dependent systems are ran a single time (mostly intended for GUI applications where
    /// certain systems should be linked to the frame rate)
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

    /// Requests a new [`Entity`] from the world.
    ///
    /// All entities must be generated this way in order to ensure they are registered properly
    /// with the world. Each entity is represented merely by an id, which is used to link it to
    /// various pieces of data stored by an associated [`Component`].
    ///
    /// [`Entity`]: crate::entity::Entity
    /// [`Component`]: crate::component::Component
    pub fn new_entity(&mut self) -> Entity {
        self.entities.push(Entity::new(self.current_id));
        self.current_id += 1;
        *self.entities.last().unwrap()
    }

    /// Registers a component with a particular [`Entity`] in the world.
    ///
    /// Stores a component so that it may be retrieved with other components of the same type and
    /// indexed by the [`Entity`]. In order for a [`Component`] to be operated on by the world, it
    /// must be registered as such. The component is stored as a [`Arc<Mutex<T>>`] under the hood.
    pub fn add_component<T: Any + Clone + 'static>(&mut self, entity: Entity, data: T) {
        self.share_component::<T>(entity, Arc::new(Mutex::new(data)));
    }

    /// Shares a component that may be used outside of the world as well.
    ///
    /// Operates the same as [`add_component`] except the [`Arc`] is generated by the user rather
    /// than the [`World`], allowing it to be accessed and modified both inside and outside the
    /// world at the same time.
    ///
    /// [`add_component`]: Self::add_component()
    pub fn share_component<T: Any + Clone + 'static>(
        &mut self,
        entity: Entity,
        data: Arc<Mutex<T>>,
    ) {
        for e in self.components.iter_mut() {
            if e.type_id == std::any::TypeId::of::<T>() {
                e.add_entity(entity, data);
                return;
            }
        }

        self.components
            .push(Component::<Arc<Mutex<dyn Any + 'static>>>::new(
                std::any::TypeId::of::<T>(),
            ));
        self.components.last_mut().unwrap().add_entity(entity, data);
    }

    /// Registers a fixed system with the world.
    ///
    /// Fixed systems operate at a fixed frequency every tick and should be used when dealing with
    /// functionality that must be deterministic. For specific information on how fixed systems are
    /// called, see [`tick`].
    ///
    /// [`tick`]: Self::tick()
    pub fn add_fixed_system<T: SystemInterface + 'a>(&mut self, system: T) {
        self.fixed_systems.push(Box::new(system));
    }

    /// Registers a dependent system with the world.
    ///
    /// Dependent systems are ran once per game tick and are intended to handle all functionality
    /// that is non-deterministic. For GUI applications, this will typically be ran once per frame.
    pub fn add_dependent_system<T: SystemInterface + 'a>(&mut self, system: T) {
        self.dependent_systems.push(Box::new(system));
    }
}

/// A helper struct to generate a [`World`].
///
/// Primarily used for circumstances in which another struct should generate the world, which might
/// take some time. This allows for the world to be setup before beginning the timer for the first
/// tick's timestep.
pub struct WorldBuilder {
    frequency: u16,
}

impl Default for WorldBuilder {
    /// Creates a world with a fixed frequency of 60 Hz.
    fn default() -> Self {
        WorldBuilder { frequency: 60 }
    }
}

impl WorldBuilder {
    /// Creates a new world with a fixed frequency of 60 Hz.
    pub fn new() -> Self {
        WorldBuilder::default()
    }

    /// Updates the fixed frequency to a selected value.
    ///
    /// The frequency represents the number of times per second fixed systems should be called.
    pub fn with_frequency(mut self, frequency: u16) -> Self {
        self.frequency = frequency;
        self
    }

    /// Generates a new world based on the prior configuration.
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

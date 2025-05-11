use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    camera::Camera,
    entity::Entity,
    graphics::{Renderer, WgpuRenderer},
    input::InputManager,
    mesh::{Model, Instance},
    system::ResourcedSystem,
    World,
};

pub struct App<'a, C, R = WgpuRenderer<'a>>
where
    R: Renderer<'a>,
    C: Camera + 'a,
{
    window: Option<Arc<Window>>,
    pub renderer: Option<Arc<Mutex<R>>>,
    pub world: World<'a>,
    keys: Arc<Mutex<HashSet<winit::keyboard::PhysicalKey>>>,
    camera: Arc<Mutex<C>>,
}

impl<'a, C: Camera + 'a> App<'a, C> {
    pub fn new(world: World<'a>, camera: Arc<Mutex<C>>) -> Self {
        Self {
            window: None,
            world,
            renderer: None,
            camera,
            keys: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Couldn't create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(&mut self).expect("Couldn't run app");
    }
}

impl<'a, C: Camera + 'a> ApplicationHandler for App<'a, C> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        let size = self.window.as_ref().unwrap().inner_size();
        self.camera
            .lock()
            .expect("no camera access")
            .resize(size.width as f32, size.height as f32);

        self.renderer = Some(Arc::new(Mutex::new(futures::executor::block_on(
            WgpuRenderer::new(self.window.clone().unwrap(), self.camera.clone()),
        ))));

        self.world.add_dependent_system(ResourcedSystem::new(
            vec![std::any::TypeId::of::<Model>(), std::any::TypeId::of::<Instance>()],
            self.renderer.as_mut().unwrap().clone(),
            |mut query, renderer| {
                let mut models = Vec::new();
                let mut instances = Vec::new();

                query.get::<Model>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(_, model)| {
                        models.push(model.clone());
                    })
                });

                query.get::<Instance>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(_, instance)| {
                        instances.push(instance.clone()); // TODO: don't clone
                    })
                });

                models.into_iter().zip(instances.drain(..)).for_each(|(model, instance)| {
                    renderer.lock().unwrap().attach(model.as_ref(), instance.as_ref().clone());
                });
            },
        ));

        self.world.add_fixed_system(ResourcedSystem::new(
            vec![std::any::TypeId::of::<InputManager>()],
            self.keys.clone(),
            |mut query, keys| {
                let mut new_keys = HashMap::<Entity, InputManager>::new();

                query.get::<InputManager>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(entity, _)| {
                        new_keys.insert(
                            entity.clone(),
                            InputManager {
                                keys: keys.lock().unwrap().clone(),
                            },
                        );
                    })
                });

                new_keys.drain().for_each(|(entity, input_manager)| {
                    query.set::<InputManager>(entity, input_manager);
                });
            },
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.world.tick();

                self.renderer.as_mut().unwrap().lock().unwrap().render();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                self.renderer
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .resize(physical_size);
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if !is_synthetic && !event.repeat {
                    if event.state == winit::event::ElementState::Pressed {
                        self.keys.lock().unwrap().insert(event.physical_key);
                    } else if event.state == winit::event::ElementState::Released {
                        self.keys.lock().unwrap().remove(&event.physical_key);
                    }
                }
            }
            _ => (),
        }
    }
}

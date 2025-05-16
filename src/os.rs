use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    camera::Camera,
    entity::Entity,
    graphics::{Renderer, WgpuRenderer},
    input::InputManager,
    mesh::{Instance, Model},
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
    input: Arc<Mutex<InputManager>>,
    camera: Arc<Mutex<C>>,
}

impl<'a, C: Camera + 'a> App<'a, C> {
    pub fn new(world: World<'a>, camera: Arc<Mutex<C>>) -> Self {
        Self {
            window: None,
            world,
            renderer: None,
            camera,
            input: Arc::new(Mutex::new(InputManager::new())),
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
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        window.set_cursor_visible(false);

        self.window = Some(Arc::new(window));

        let size = self.window.as_ref().unwrap().inner_size();
        self.camera
            .lock()
            .expect("no camera access")
            .resize(size.width as f32, size.height as f32);

        self.renderer = Some(Arc::new(Mutex::new(futures::executor::block_on(
            WgpuRenderer::new(self.window.clone().unwrap(), self.camera.clone()),
        ))));

        self.window.as_mut().unwrap()
            .set_cursor_grab(winit::window::CursorGrabMode::Locked);

        self.world.add_dependent_system(ResourcedSystem::new(
            vec![
                std::any::TypeId::of::<Model>(),
                std::any::TypeId::of::<Instance>(),
            ],
            self.renderer.as_mut().unwrap().clone(),
            |mut query, renderer| {
                // TODO: fix
                /*query.get::<Model>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(_, model)| {
                        models.push(model.clone());
                    })
                });

                query.get::<Instance>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(_, instance)| {
                        instances.push(instance.clone()); // TODO: don't clone
                    })
                });*/

                /*models
                .into_iter()
                .zip(instances.drain(..))
                .for_each(|(model, instance)| {
                    renderer
                        .lock()
                        .unwrap()
                        .attach(model.as_ref(), instance.as_ref().clone());
                });*/
            },
        ));

        self.world.add_fixed_system(ResourcedSystem::new(
            vec![std::any::TypeId::of::<InputManager>()],
            self.input.clone(),
            |mut query, input| {
                let mut new_keys = HashMap::<Entity, InputManager>::new();

                /*query.get::<InputManager>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(entity, _)| {
                        new_keys.insert(entity.clone(), input.lock().unwrap().clone());
                    })
                });*/

                input.lock().unwrap().analog_input = (0.0, 0.0);

                new_keys.drain().for_each(|(entity, input_manager)| {
                    //query.set::<InputManager>(entity, input_manager);
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
                        self.input.lock().unwrap().keys.insert(event.physical_key);
                    } else if event.state == winit::event::ElementState::Released {
                        self.input.lock().unwrap().keys.remove(&event.physical_key);
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state: _,
                button: _,
            } => {
                // TODO: handle mouse input
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        // TODO: ensure wayland gets mouse events
        match event {
            DeviceEvent::Motion { axis, value } => {
                // TODO: is there a better way to do this?
                match axis {
                    0 => {
                        self.input.lock().unwrap().analog_input.0 = value as f32;
                    }
                    1 => {
                        self.input.lock().unwrap().analog_input.1 = value as f32;
                    }
                    _ => {
                        panic!("unknown axis");
                    }
                }
            }
            _ => {}
        }
    }
}

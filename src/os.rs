use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use std::sync::{Arc, Mutex};

use crate::{
    graphics::{Renderer, WgpuRenderer},
    mesh::Model,
    system::ResourcedSystem,
    World,
};

pub struct App<'a, R = WgpuRenderer<'a>>
where
    R: Renderer<'a>,
{
    window: Option<Arc<Window>>,
    pub renderer: Option<Arc<Mutex<R>>>,
    pub world: World<'a>,
    phantom: std::marker::PhantomData<&'a R>,
    init_mesh: u32
}

impl<'a> App<'a> {
    pub fn new(world: World<'a>, init_mesh: u32) -> Self {
        Self {
            window: None,
            world,
            renderer: None,
            phantom: std::marker::PhantomData,
            init_mesh
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Couldn't create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(&mut self).expect("Couldn't run app");
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));

        self.renderer = Some(Arc::new(Mutex::new(futures::executor::block_on(
            WgpuRenderer::new(self.window.clone().unwrap(), self.init_mesh),
        ))));

        self.world.add_dependent_system(ResourcedSystem::new(
            vec![std::any::TypeId::of::<Model>()],
            self.renderer.as_mut().unwrap().clone(),
            |mut query, renderer| {
                query.get::<Model>().iter_mut().for_each(|e| {
                    e.data.iter_mut().for_each(|(_, model)| {
                        renderer.lock().unwrap().attach(model.as_ref());
                    })
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
            _ => (),
        }
    }
}

use aspen::{
    camera::FpvCamera,
    entity::Entity,
    input::InputManager,
    mesh::{Instance, Model},
    system::{Query, System},
    App, WorldBuilder,
};

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use winit::keyboard::KeyCode;

#[derive(Clone, Debug)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    let mut world = WorldBuilder::new().with_frequency(60).build();

    let balls = std::iter::repeat(0)
        .take(9)
        .map(|_| world.new_entity())
        .collect::<Vec<Entity>>();

    let sphere_model = Model::from_obj("sphere.obj");
    balls.iter().for_each(|ball| {
        world.add_component(*ball, sphere_model.clone());
    });

    let input_manager = world.new_entity();
    world.add_component(
        input_manager,
        InputManager {
            keys: HashSet::new(),
        },
    );

    let camera = Arc::new(Mutex::new(FpvCamera {
        eye: nalgebra::Point3::new(2.0, 3.0, 4.0),
        target: nalgebra::Point3::new(0.0, 0.0, 0.0),
        up: nalgebra::Vector3::y(),
        fovy: 45.0,
        ..Default::default()
    }));

    world.add_component(input_manager, camera.clone());

    balls.iter().enumerate().for_each(|(index, ball)| {
        if index < 5 {
            world.add_component(
                *ball,
                Velocity {
                    x: index as f32,
                    y: index as f32 + 1.0,
                    z: index as f32 + 2.0,
                },
            )
        }

        world.add_component(*ball, {
            let mut instance = Instance::new(&sphere_model.mesh);
            instance.translate(nalgebra::Translation3::from(nalgebra::Vector3::new(
                3.0 * (index % 3) as f32,
                3.0 * (index / 3) as f32,
                0.0,
            )));
            instance
        });
    });

    world.add_fixed_system(System::new(
        vec![
            std::any::TypeId::of::<InputManager>(),
            std::any::TypeId::of::<Arc<Mutex<FpvCamera>>>(),
        ],
        |mut query: Query| {
            let mut keys = Vec::new();

            query.get::<InputManager>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(_entity, input)| {
                    input.keys.iter().for_each(|key| {
                        keys.push(key.clone());
                    });
                });
            });

            keys.into_iter().for_each(|key| {
                query
                    .get::<Arc<Mutex<FpvCamera>>>()
                    .iter_mut()
                    .for_each(|e| {
                        e.data.iter_mut().for_each(|(_entity, camera)| {
                            if let winit::keyboard::PhysicalKey::Code(code) = key {
                                let mut camera = camera.lock().unwrap();

                                let t = nalgebra::Isometry3::new(
                                    match code {
                                        KeyCode::KeyW => camera.target - camera.eye,
                                        KeyCode::KeyS => -1.0 * (camera.target - camera.eye),
                                        KeyCode::KeyA => {
                                            -1.0 * (camera.target - camera.eye).cross(&camera.up)
                                        }
                                        KeyCode::KeyD => {
                                            (camera.target - camera.eye).cross(&camera.up)
                                        }
                                        _ => nalgebra::Vector3::zeros(),
                                    }
                                    .try_normalize(0.001.into())
                                    .unwrap_or(nalgebra::Vector3::zeros())
                                        * 0.08,
                                    nalgebra::Vector3::new(0.0, 0.0, 0.0),
                                );

                                camera.eye = t * camera.eye;
                            }
                        });
                    });
            });
        },
    ));

    world.add_fixed_system(System::new(
        vec![
            std::any::TypeId::of::<Instance>(),
            std::any::TypeId::of::<Velocity>(),
        ],
        |mut query: Query| {
            let mut new_instance: std::collections::HashMap<Entity, Instance> =
                std::collections::HashMap::new();

            query.get::<Instance>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, instance)| {
                    new_instance.insert(entity.clone(), (*instance.as_ref()).clone());
                });
            });

            query.get::<Velocity>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, velocity)| {
                    new_instance.get_mut(entity).map(|instance| {
                        instance.translate(nalgebra::Translation3::from(nalgebra::Vector3::new(
                            velocity.x / 600.0,
                            velocity.y / 600.0,
                            velocity.z / 600.0,
                        )));
                    });
                });
            });

            new_instance
                .drain()
                .for_each(|(entity, instance)| query.set::<Instance>(entity, instance))
        },
    ));

    let app = App::new(world, camera);
    app.run();
}

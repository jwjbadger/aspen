use aspen::{
    camera::Camera,
    camera::FpvCamera,
    /*component::Component,*/
    entity::Entity,
    input::InputManager,
    mesh::{Mesh, Model, Vertex, Instance},
    system::{Query, System},
    App, /*World,*/ WorldBuilder,
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
        .take(10)
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

        world.add_component(
            *ball,
            Instance::new(&sphere_model.mesh).with_transform(
                nalgebra::Translation3::from(
                    nalgebra::Vector3::new(3.0 * index as f32, 0.0, 0.0),
                ),
            ),
        );
    });

    world.add_fixed_system(System::new(
        vec![
            std::any::TypeId::of::<InputManager>(),
            std::any::TypeId::of::<Arc<Mutex<FpvCamera>>>(),
        ],
        |mut query: Query| {
            let mut keys = Vec::new();

            query.get::<InputManager>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, input)| {
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
                        e.data.iter_mut().for_each(|(entity, camera)| {
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
                                        * 0.01,
                                    nalgebra::Vector3::new(0.0, 0.0, 0.0),
                                );

                                camera.eye = t * camera.eye;
                            }
                        });
                    });
            });
        },
    ));

    /*world.add_fixed_system(System::new(
        vec![
            std::any::TypeId::of::<Instance>(),
            std::any::TypeId::of::<Velocity>(),
        ],
        |mut query: Query| {
            let mut new_transform: std::collections::HashMap<Entity, Instance> =
                std::collections::HashMap::new();

            query.get::<Transform>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, trans)| {
                    new_position.insert(
                        entity.clone(),
                        Transform {
                            position: trans.position.clone(),
                            scale: trans.scale.clone(),
                            rotation: trans.rotation.clone(),
                        }
                    );
                });
            });

            query.get::<Velocity>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, velocity)| {
                    new_transform.get_mut(entity).map(|trans| {
                        trans.position.x += velocity.x / 60.0;
                        trans.position.y += velocity.y / 60.0;
                        trans.position.z += velocity.z / 60.0;
                    });
                });
            });

            new_transform
                .drain()
                .for_each(|(entity, new_trans)| query.set::<Transform>(entity, new_trans))
        },
    ));*/

    let app = App::new(world, camera);
    app.run();
}

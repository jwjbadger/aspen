use aspen::{
    camera::FlyCamera,
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
    world.add_component(input_manager, InputManager::new());

    let camera = Arc::new(Mutex::new(FlyCamera {
        eye: nalgebra::Point3::new(2.0, 3.0, 4.0),
        dir: nalgebra::Vector3::x(),
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
            std::any::TypeId::of::<Arc<Mutex<FlyCamera>>>(),
        ],
        |mut query: Query| {
            /*let mut keys = Vec::new();
            let mut analog_input: (f32, f32) = (0.0, 0.0);
            let mut entities = Vec::new();

            query.get::<InputManager>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, input)| {
                    entities.push(entity.clone());

                    input.keys.iter().for_each(|key| {
                        keys.push(key.clone());
                    });
                    analog_input = input.analog_input;
                });
            });

            query
                .get::<Arc<Mutex<FlyCamera>>>()
                .iter_mut()
                .for_each(|e| {
                    e.data.iter_mut().for_each(|(_entity, camera)| {
                        let mut camera = camera.lock().unwrap();
                        let up = camera.up;
                        let right = camera.up.cross(&camera.dir);

                        camera.turn(
                            nalgebra::UnitQuaternion::from_axis_angle(
                                &nalgebra::Unit::new_normalize(up),
                                -1.0 * analog_input.0 * 0.0008,
                            ) * nalgebra::UnitQuaternion::from_axis_angle(
                                &nalgebra::Unit::new_normalize(right),
                                analog_input.1 * 0.0008,
                            ),
                        );

                        keys.iter().for_each(|key| {
                            if let winit::keyboard::PhysicalKey::Code(code) = key {
                                let t = nalgebra::Isometry3::new(
                                    match code {
                                        KeyCode::KeyW => camera.dir,
                                        KeyCode::KeyS => -1.0 * camera.dir,
                                        KeyCode::KeyA => -1.0 * (camera.dir).cross(&camera.up),
                                        KeyCode::KeyD => (camera.dir).cross(&camera.up),
                                        KeyCode::ShiftLeft => camera.up,
                                        KeyCode::ControlLeft => -1.0 * camera.up,
                                        _ => nalgebra::Vector3::zeros(),
                                    }
                                    .try_normalize(0.001.into())
                                    .unwrap_or(nalgebra::Vector3::zeros())
                                        * 0.08,
                                    nalgebra::Vector3::new(0.0, 0.0, 0.0),
                                );

                                camera.eye = t.transform_point(&camera.eye);
                            }
                        });
                    });
                });

            entities.drain(..).for_each(|e| {
                query.set::<InputManager>(
                    e,
                    InputManager {
                        keys: HashSet::from_iter(keys.clone().into_iter()),
                        analog_input: (0.0, 0.0),
                    },
                );
            });*/
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

            /*query.get::<Instance>().iter_mut().for_each(|e| {
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
                .for_each(|(entity, instance)| query.set::<Instance>(entity, instance))*/
        },
    ));

    let app = App::new(world, camera);
    app.run();
}

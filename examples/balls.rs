use aspen::{
    camera::FpvCamera,
    /*component::Component,*/
    entity::Entity,
    mesh::{Mesh, Model, Vertex},
    input::InputManager,
    system::{Query, System},
    App, /*World,*/ WorldBuilder,
};

use std::collections::HashSet;

#[derive(Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Debug)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    let mut world = WorldBuilder::new().with_frequency(60).build();

    let balls = std::iter::repeat(0)
        .take(2)
        .map(|_| world.new_entity())
        .collect::<Vec<Entity>>();

    balls.iter().for_each(|ball| {
        world.add_component(
            *ball,
            Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
    });

    let input_manager = world.new_entity();
    world.add_component(
        input_manager,
        InputManager {
            keys: HashSet::new(),
        }
    );

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
            Model {
                mesh: Mesh::new(if index == 0 {
                    vec![
                        Vertex {
                            position: [-0.5, 0.5, 0.0],
                            color: [1.0, 0.0, 0.0],
                        },
                        Vertex {
                            position: [-0.5, -0.5, 0.0],
                            color: [0.0, 1.0, 0.0],
                        },
                        Vertex {
                            position: [0.5, -0.5, 0.0],
                            color: [0.0, 0.0, 1.0],
                        },
                    ]
                } else {
                    vec![
                        Vertex {
                            position: [-0.5, 0.5, 0.0],
                            color: [1.0, 0.0, 0.0],
                        },
                        Vertex {
                            position: [0.5, -0.5, 0.0],
                            color: [0.0, 0.0, 1.0],
                        },
                        Vertex {
                            position: [0.5, 0.5, 0.0],
                            color: [0.0, 1.0, 0.0],
                        },
                    ]
                }),
            },
        );
    });

    world.add_fixed_system(System::new(
        vec![std::any::TypeId::of::<InputManager>()],
        |mut query: Query| {
            query.get::<InputManager>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, input)| {
                    println!("{:?}", input);
                });
            });
        },
    ));

    world.add_fixed_system(System::new(
        vec![
            std::any::TypeId::of::<Position>(),
            std::any::TypeId::of::<Velocity>(),
        ],
        |mut query: Query| {
            let mut new_position: std::collections::HashMap<Entity, Position> =
                std::collections::HashMap::new();

            query.get::<Position>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, pos)| {
                    new_position.insert(
                        entity.clone(),
                        Position {
                            x: pos.x,
                            y: pos.y,
                            z: pos.z,
                        },
                    );
                });
            });

            query.get::<Velocity>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(entity, velocity)| {
                    new_position.get_mut(entity).map(|pos| {
                        pos.x += velocity.x / 60.0;
                        pos.y += velocity.y / 60.0;
                        pos.z += velocity.z / 60.0;
                    });
                });
            });

            new_position
                .drain()
                .for_each(|(entity, new_pos)| query.set::<Position>(entity, new_pos))
        },
    ));

    let app = App::new(
        world,
        FpvCamera {
            eye: nalgebra::Point3::new(2.0, 3.0, 4.0),
            target: nalgebra::Point3::new(0.0, 0.0, 0.0),
            up: nalgebra::Vector3::y(),
            fovy: 45.0,
            ..Default::default()
        },
    );
    app.run();
}

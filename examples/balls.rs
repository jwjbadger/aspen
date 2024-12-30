use aspen::{
    component::Component,
    entity::Entity,
    system::{Query, System},
    World, WorldBuilder,
};

#[derive(Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    let mut world = WorldBuilder::new().timed_loop(60).build();
    let ball = world.new_entity();
    world.add_component(
        ball,
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );
    world.add_fixed_system(System::new(
        vec![std::any::TypeId::of::<Position>()],
        |mut query: Query| {
            let mut to_set: (Entity, Position) = (Entity::new(0), Position { x: 0.0, y: 0.0, z: 0.0 });

            query.get::<Position>().iter_mut().for_each(|e| {
                e.data.iter_mut().for_each(|(key, value)| {
                    println!("{:#?}: {:#?}", key, value.x);
                    to_set.0 = key.clone();
                    to_set.1 = Position {
                        x: value.x + (1.0 / 60.0),
                        y: value.y,
                        z: value.z
                    };
                });
            });

            query.set::<Position>(
                to_set.0,
                to_set.1
            )
        },
    ));

    world.run();
}

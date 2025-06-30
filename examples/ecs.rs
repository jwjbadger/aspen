use aspeng::ecs::{WorldBuilder, entity::Entity, system::{System, FixedSystem}, query::Query};

struct Position {
    x: f32,
    y: f32,
    z: f32
} 

struct Velocity {
    x: f32,
    y: f32,
    z: f32
}

struct Name(String);

fn main() {
    let world_builder = WorldBuilder::new(60); 

    let objects = std::iter::repeat(0).take(10).map(|_| Entity::new()).collect::<Vec<_>>();
    objects.iter().for_each(|obj| {
        world_builder.add_component(obj, Position { x: 0.0, y: 0.0, z: 0.0 });
        world_builder.add_component(obj, Velocity { x: 0.0, y: 0.0, z: 0.0 });
    });

    world_builder.add_component(&objects[0], Name(String::new("First Entity!")));

    world_builder.register_system(FixedSystem::new(|query: Query<(&mut Position, &Velocity)>| {
        query.for_each(|(&mut position, velocity)| {
            position.x += velocity.x;
            position.y += velocity.y;
            position.z += velocity.z;
        });
    }));

    world_builder.register_system(System::new(|query: Query<(&Name)>| {
        query.for_each(|(&name)| {
            println!("{}", name);
        });
    }));

    let world = world_builder.build();

    loop {
        world.tick();
    }
}

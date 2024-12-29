use aspen::{World, WorldBuilder};

fn main() {
    let mut world = WorldBuilder::new().timed_loop(60).build();
    world.run();
}

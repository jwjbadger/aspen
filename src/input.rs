use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InputManager {
    pub keys: HashSet<winit::keyboard::PhysicalKey>,
    pub analog_input: (f32, f32),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            analog_input: (0.0, 0.0),
        }
    }
}

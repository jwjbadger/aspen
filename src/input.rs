use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct InputManager {
    pub keys: HashSet<winit::keyboard::PhysicalKey>,
}

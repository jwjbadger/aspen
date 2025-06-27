use std::collections::HashSet;

/// The main access point to user input for GUI applications
///
/// Should be added as a component to a single entity and used to get input from the application.
/// On every frame, the application will push input to the data stored in this struct to be
///ihandled by a custom system. 
#[derive(Debug, Clone)]
pub struct InputManager {
    /// Contains all the keys pressed between frames.
    pub keys: HashSet<winit::keyboard::PhysicalKey>,
    /// Contains any analog movement betwen frames. Reset every frame as a delta around (0, 0).
    pub analog_input: (f32, f32),
}

impl InputManager {
    /// Creates a new empty input manager.
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            analog_input: (0.0, 0.0),
        }
    }
}

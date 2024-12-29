use std::collections::HashSet;
use std::any::TypeId;

// Somehow has to match typeid to component
pub struct Query;

pub struct System {
    components: HashSet<TypeId>,
    executable: fn(Query),
}

impl System {
    fn new(components: HashSet<TypeId>, executable: fn(Query)) -> Self {
        Self {
            components,
            executable 
        }
    }

    fn execute(&self, query: Query) {
        (self.executable)(query) 
    }
}

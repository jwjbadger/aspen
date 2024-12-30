use std::collections::HashSet;
use std::any::TypeId;
use crate::component::Component;
use std::rc::Rc;

// Somehow has to match typeid to component
pub struct Query {
    matches: Vec::<Rc<dyn Component>>,
}

impl Query{
    pub fn new(haystack: Vec::<Rc<dyn Component>>, filter: &HashSet<TypeId>) -> Self {

        Self {
            matches: haystack.into_iter().filter(|e| filter.contains(&(*e).type_id())).collect()
        }
    }

    pub fn get<T: 'static>(&mut self) -> Vec::<Rc<dyn Component>> {
        self.matches.clone().into_iter().filter(|e| e.type_id() == TypeId::of::<T>()).collect() 
    }
}


pub struct System {
    pub components: HashSet<TypeId>,
    pub executable: fn(Query),
}

impl System {
    fn new(components: HashSet<TypeId>, executable: fn(Query)) -> Self {
        Self {
            components,
            executable 
        }
    }

    pub fn execute(&self, query: Query) {
        (self.executable)(query) 
    }
}

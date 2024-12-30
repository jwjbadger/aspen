use crate::Entity;

pub trait Component: std::any::Any {
    fn entities(&self) -> Vec<Entity>;
}

use crate::Entity;

pub trait Component {
    fn entities(&self) -> Vec<Entity>;
}

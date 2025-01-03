pub trait Renderer {
    fn attach<T>(&mut self, item: T)
    where
        T: Renderable;
    fn render(&mut self);
}

pub trait Renderable {}

pub struct WgpuRenderer;

impl Renderer for WgpuRenderer {
    fn attach<T>(&mut self, _item: T)
    where
        T: Renderable {
        todo!()
    }

    fn render(&mut self) {
        todo!()
    }
}

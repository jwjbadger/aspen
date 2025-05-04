use crate::graphics::Renderable;
use bytemuck::NoUninit;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
}

impl Renderable for Mesh {
    fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x3,
            ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

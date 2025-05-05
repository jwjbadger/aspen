use crate::graphics::Renderable;
use bytemuck::NoUninit;

#[derive(Clone, Debug)]
pub struct Model {
    pub mesh: Mesh,
}

impl Renderable for Model {
    fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub(crate) id: Option<MeshId>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertices,
            id: None,
        }
    }

    pub fn id(mut self, id: u32) -> Mesh {
        self.id = Some(MeshId(id));
        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
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

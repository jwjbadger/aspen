use crate::graphics::Renderable;
use bytemuck::NoUninit;
use std::sync::atomic::{AtomicU32, Ordering};
use std::io::{BufReader, Cursor};

static MESHES: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub struct Model {
    pub mesh: Mesh,
}

fn load_res(file_name: &str) -> String {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    std::fs::read_to_string(&path).expect(&format!("Failed to read file: {:#?}", &path))
}

impl Model {
    pub fn from_obj(file_name: &str) -> Self {
        let obj_text = load_res(file_name);

        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, mtls) = tobj::load_obj_buf(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            move |p| {
                let mat_text = load_res(p.to_str().unwrap());
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        ).unwrap();

        let mut meshes = models.into_iter().map(|m| {
            Mesh::new((0..m.mesh.positions.len() / 3).map(|i| {
                Vertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    color: [
                        0.0,
                        0.0,
                        0.0,
                    ],
                }  
            }).collect::<Vec<_>>())
        }).collect::<Vec<_>>();

        Self {
            mesh: meshes.pop().unwrap(), // TODO: handle multiple meshes
        }
    }
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
    pub(crate) id: MeshId,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        // TOOO: error if vertices is empty or not triangles
        Self {
            vertices,
            id: MeshId(MESHES.fetch_add(1, Ordering::SeqCst)),
        }
    }
}

pub struct MeshInfo {
    pub vertex_count: u32,
    pub vertex_buffer: wgpu::Buffer,
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

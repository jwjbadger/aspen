use crate::{graphics::Renderable, texture::TextureBuilder};
use bytemuck::NoUninit;
use std::io::{BufReader, Cursor};
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu::util::DeviceExt;

static MESHES: AtomicU32 = AtomicU32::new(0);
static INSTANCES: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub(crate) struct InstanceInfo {
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) instance_buffer_size: usize,
    pub(crate) instance_count: usize,
    instances: Vec<Instance>,
}

impl InstanceInfo {
    pub(crate) fn new(device: &wgpu::Device, instances: Vec<Instance>) -> Self {
        let instance_count = instances.len();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            instance_buffer,
            instance_buffer_size: instance_count, // TODO: vec-like resizing
            instance_count,
            instances,
        }
    }

    pub(crate) fn append(&mut self, device: &wgpu::Device, instance: Instance) {
        self.instances.push(instance);
        let instance_data = self
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // TODO: destroy?

        self.instance_buffer = instance_buffer;
        self.instance_count += 1;
        self.instance_buffer_size += 1;
    }

    pub(crate) fn remove(&mut self, id: InstanceId) {
        self.instances.retain(|instance| instance.id != id);
        self.instance_count -= 1;
        // TODO: remove instance from buffer
    }

    pub(crate) fn contains(&self, id: InstanceId) -> bool {
        self.instances.iter().any(|instance| instance.id == id)
    }
}

/// An instance of a particular mesh.
///
/// Defines details about the position and rotation to take advantage of instanced rendering in
/// renderers that implement such functionality. Refers to a mesh that it describes.
#[derive(Clone, Debug)]
pub struct Instance {
    /// The position of the instance.
    pub translation: nalgebra::Translation3<f32>,
    /// The scale of the instance.
    pub scale: nalgebra::Scale3<f32>,
    /// The rotation of the instance.
    pub rotation: nalgebra::UnitQuaternion<f32>,
    /// The id of the mesh that is referenced by the given instance.
    pub mesh_id: MeshId,
    pub(crate) id: InstanceId,
}

impl Instance {
    /// Creates a new instance referencing a given mesh with no transformations.
    pub fn new(mesh: &Mesh) -> Self {
        Self {
            translation: nalgebra::Translation3::identity(),
            scale: nalgebra::Scale3::identity(),
            rotation: nalgebra::UnitQuaternion::identity(),
            mesh_id: mesh.id,
            id: InstanceId(INSTANCES.fetch_add(1, Ordering::SeqCst)),
        }
    }

    /// Translates the instance by the given amount.
    ///
    /// Operates as a delta off of the previous position of the instance.
    pub fn translate(&mut self, translate: nalgebra::Translation3<f32>) {
        self.translation *= translate;
    }

    /// Scales the instance by the given amount.
    ///
    /// Operates as an absolute without reference to the previous scale of the instance.
    pub fn scale(&mut self, scale: nalgebra::Scale3<f32>) {
        self.scale = scale;
    }

    /// Rotates the instance by the given amount.
    ///
    /// Operates as a delta off of the previous rotation.
    pub fn rotate(&mut self, rotation: nalgebra::UnitQuaternion<f32>) {
        self.rotation *= rotation;
    }

    pub(crate) fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (self.translation.to_homogeneous()
                * self.rotation.to_homogeneous()
                * self.scale.to_homogeneous())
            .into(),
        }
    }
}

/// A model that represent the combination of everything needed to display a given object.
#[derive(Clone, Debug)]
pub struct Model {
    /// The mesh around which everything else is built.
    pub mesh: Mesh,
    /// Describes what the texture for the mesh should look like. Should be built by the renderer
    /// when attached for the first time.
    pub texture_builder: Option<TextureBuilder>,
}

fn load_res(file_name: &str) -> String {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    std::fs::read_to_string(&path).expect(&format!("Failed to read file: {:#?}", &path))
}

impl Model {
    /// Adds an optional texture to the mesh.
    pub fn with_tex(mut self, builder: TextureBuilder) -> Self {
        self.texture_builder = Some(builder);
        self
    }

    /// Creates a mesh from an obj file.
    ///
    /// The obj file should be located in the res folder, which will need to be in the build
    /// directory of the program since the filename is relative to the build directory.
    pub fn from_obj(file_name: &str) -> Self {
        let obj_text = load_res(file_name);

        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, _mtls) = tobj::load_obj_buf(
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
        )
        .unwrap();

        let mut meshes = models
            .into_iter()
            .map(|m| {
                // TODO: use indexed drawing
                Mesh::new(
                    m.mesh
                        .indices
                        .iter()
                        .map(|&i| Vertex {
                            position: [
                                m.mesh.positions[i as usize * 3],
                                m.mesh.positions[i as usize * 3 + 1],
                                m.mesh.positions[i as usize * 3 + 2],
                            ],
                            tex_coords: [m.mesh.texcoords[(i * 2) as usize], 1.0 - m.mesh.texcoords[(i * 2 + 1) as usize]],
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        if meshes.len() > 1 {
            panic!("can't do multiple meshes");
        }

        Self {
            mesh: meshes.pop().unwrap(), // TODO: handle multiple meshes
            texture_builder: None
        }
    }
}

impl Renderable for Model {
    fn tex_builder(&self) -> Option<TextureBuilder> {
        self.texture_builder.clone()
    }

    fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

pub(crate) struct ModelInfo {
    pub(crate) mesh_info: MeshInfo,
    pub(crate) texture_bind_group: Option<wgpu::BindGroup>
}

/// A newtype of [`u32`] that represents the id of a given mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshId(pub u32);

/// A newtype of [`u32`] that represents the id of a given instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u32);

/// The raw vertices of a given object describing what it should look like when actually rendered.
#[derive(Clone, Debug)]
pub struct Mesh {
    /// The raw vertex coordinates for the mesh.
    pub vertices: Vec<Vertex>,
    pub(crate) id: MeshId,
}

impl Mesh {
    /// Creates a new mesh given its vertices.
    ///
    /// Because vertices are currently not indexed and the mesh is expected to be triangulated,
    /// there must be a multiple of three vertices in this struct. Vertices will be connected such
    /// that each pair of three forms a triangle.
    pub fn new(vertices: Vec<Vertex>) -> Self {
        // TOOO: error if vertices is empty or not triangles
        Self {
            vertices,
            id: MeshId(MESHES.fetch_add(1, Ordering::SeqCst)),
        }
    }
}

pub(crate) struct MeshInfo {
    pub(crate) vertex_count: u32,
    pub(crate) vertex_buffer: wgpu::Buffer,
}

/// A particular point in space representing a corner of a mesh with associated data.
#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit)]
pub struct Vertex {
    /// The position of the vertex in the format [x, y, z].
    pub position: [f32; 3],
    /// The coordinates of the texture that should be applied to this point.
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2,
    ];

    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

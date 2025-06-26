use crate::{
    camera::{Camera, CameraUniform},
    texture::TextureBuilder,
    mesh::{Instance, InstanceInfo, InstanceRaw, Mesh, MeshId, MeshInfo, ModelInfo, Vertex},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

pub trait Renderer<'a> {
    fn attach<T>(&mut self, item: &T, instance: Instance)
    where
        T: Renderable;
    fn render(&mut self);
    fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>);
}

pub trait Renderable {
    fn tex_builder(&self) -> Option<TextureBuilder>;
    fn mesh(&self) -> &Mesh;
}

pub struct WgpuRenderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    render_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    vertex_buffers: HashMap<MeshId, ModelInfo>,
    instances: HashMap<MeshId, InstanceInfo>,
    camera: Arc<Mutex<dyn Camera + 'a>>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl<'a> Renderer<'a> for WgpuRenderer<'a> {
    fn attach<T>(&mut self, item: &T, instance: Instance)
    where
        T: Renderable,
    {
        if self.vertex_buffers.get(&item.mesh().id).is_none() {
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&item.mesh().vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            self.vertex_buffers.insert(
                item.mesh().id,
                ModelInfo {
                    mesh_info: MeshInfo {
                        vertex_count: item.mesh().vertices.len() as u32,
                        vertex_buffer,
                    },
                    texture: item.tex_builder().map(|builder| builder.build(&self.device, &self.queue, &self.texture_bind_group_layout)),
                },
            );
        }

        if self.instances.get(&item.mesh().id).is_none() {
            self.instances
                .insert(item.mesh().id, InstanceInfo::new(&self.device, vec![]));
        } else if self
            .instances
            .get(&item.mesh().id)
            .unwrap()
            .contains(instance.id)
        {
            self.instances
                .get_mut(&item.mesh().id)
                .unwrap()
                .remove(instance.id);
        }

        self.instances
            .get_mut(&item.mesh().id)
            .unwrap()
            .append(&self.device, instance);
        // TODO: refactor to remove unused instances
    }

    fn render(&mut self) {
        self.camera_uniform
            .update_raw(self.camera.lock().unwrap().build_view_projection_matrix());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let current_texture = self.surface.get_current_texture().unwrap();
        let view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Aspen Command Encoder"),
                });

        {
            let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Aspen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for mesh in self.instances.keys() {
                let model_info = self.vertex_buffers.get(mesh).expect("Mesh not found");
                let instance_info = self.instances.get(mesh).expect("Instance not found");

                if let Some(tex) = model_info.texture.as_ref() {
                    pass.set_bind_group(1, Some(&tex.diffuse_bind_group), &[]); 
                } else {
                    panic!("No texture")
                }

                pass.set_vertex_buffer(0, model_info.mesh_info.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, instance_info.instance_buffer.slice(..));
                pass.draw(
                    0..model_info.mesh_info.vertex_count,
                    0..instance_info.instance_count as u32,
                ); // TODO: Use the actual vertex count
            }

            self.vertex_buffers.clear();
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        current_texture.present();
    }

    fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = physical_size.width;
        self.surface_config.height = physical_size.height;

        let mut camera = self.camera.lock().unwrap();
        camera.resize(physical_size.width as f32, physical_size.height as f32);

        self.camera_uniform
            .update_raw(camera.build_view_projection_matrix());

        self.surface.configure(&self.device, &self.surface_config);
    }
}

impl<'a> WgpuRenderer<'a> {
    pub async fn new(
        window: Arc<winit::window::Window>,
        camera: Arc<Mutex<impl Camera + 'a>>,
    ) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Aspen Device"),
                required_features: wgpu::Features::empty(),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&(*camera.lock().unwrap()));

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Aspen Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Aspen Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Aspen Vertex Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
                }),
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Aspen Fragment Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
                }),
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        });

        WgpuRenderer {
            surface,
            device,
            render_pipeline,
            queue,
            surface_config: config,
            vertex_buffers: HashMap::new(),
            instances: HashMap::new(),
            camera,
            camera_buffer,
            camera_uniform,
            camera_bind_group,
            texture_bind_group_layout,
        }
    }
}

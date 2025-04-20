use std::sync::Arc;

pub trait Renderer<'a> {
    fn attach<T>(&mut self, item: &T)
    where
        T: Renderable;
    fn render(&mut self);
    fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>);
}

pub trait Renderable {}

#[derive(Clone, Debug)]
pub struct Mesh;
impl Renderable for Mesh {}

pub struct WgpuRenderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    render_pipeline: wgpu::RenderPipeline,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<'a> Renderer<'a> for WgpuRenderer<'a> {
    fn attach<T>(&mut self, _item: &T)
    where
        T: Renderable,
    {
        println!("attaching...")
    }

    fn render(&mut self) {
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        current_texture.present();
    }

    fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = physical_size.width;
        self.surface_config.height = physical_size.height;

        self.surface.configure(&self.device, &self.surface_config);
    }
}

impl<'a> WgpuRenderer<'a> {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Aspen Device"),
                    required_features: wgpu::Features::empty(),
                    ..Default::default()
                },
                None,
            )
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Aspen Render Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Aspen Vertex Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
                }),
                entry_point: Some("vs_main"),
                buffers: &[],
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
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        }
    }
}

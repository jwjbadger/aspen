/// A WGPU texture representing the underlying data used by WGPU to actually attach the texture to
/// an object.
pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler
}

impl Texture {
    pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a bind group from the underlying texture data.
    ///
    /// Should only be used internally or when creating a new renderer.
    ///
    /// Used to turn a texture into a bind group, which allows WGPU to actually make use of the
    /// texture when rendering a mesh. Used by nearly every texture except for the depth texture.
    pub fn into_bind_group(self, device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        })
    }

    /// Creates a depth texture.
    ///
    /// Should only be used internally or when creating a new renderer.
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("Aspen Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler } 
    }
}

/// Represents the raw data that will eventually be passed off to WGPU describing a texture.
///
/// For now, only holds raw image data describing the texture that will be applied to a given
/// object, which can then be used to build the WGPU structures needed to apply that structure to
/// an object when the builder is passed off to the renderer.
#[derive(Clone, Debug)]
pub struct TextureBuilder {
    image: image::RgbaImage
}

impl TextureBuilder {
    /// Generates a [`TextureBuilder`] from an image which is placed in the res folder in the build
    /// directory.
    pub fn from_image(filename: &str) -> Self {
        let diffuse_image = image::ImageReader::open(std::path::Path::new(env!("OUT_DIR")).join("res").join(filename)).unwrap().decode().unwrap();

        Self {
            image: diffuse_image.to_rgba8()
        }
    }

    /// Builds the texture from the given configuration.
    ///
    /// Should only be used internally or when creating a new renderer.
    pub fn build(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
        let dimensions = self.image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture =
            device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                    label: Some("diffuse_texture"),
                    view_formats: &[],
            });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler =
            device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });


        Texture {
            texture: diffuse_texture,
            view: diffuse_texture_view,
            sampler: diffuse_sampler
        }
    }
}



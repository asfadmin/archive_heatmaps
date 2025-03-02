use image::GenericImageView;
use web_sys::js_sys::Math::ceil;

pub struct TextureContext {
    pub texture: wgpu::Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

/// Generate a texture that the blend render pass can render to and the colormap render pass can read from
pub fn generate_blend_texture(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> TextureContext {
    let blend_texture_size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    // Create a 2D R16Float texture with appropriate usages
    let blend_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: blend_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("blend texture"),
        view_formats: &[],
    });

    // Set up a sampler for the texture we just created
    let blend_texture_view = blend_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let blend_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Set up the bind group for the above texture and sampler
    let blend_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
            label: Some("blend_bind_group_layout"),
        });

    let blend_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &blend_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&blend_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&blend_sampler),
            },
        ],
        label: Some("blend_bind_group"),
    });

    TextureContext {
        texture: blend_texture,
        bind_group_layout: blend_bind_group_layout,
        bind_group: blend_bind_group,
    }
}

/// Generates 2 1D texture with the colormaps the heatmap will use
pub fn generate_colormaps(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (TextureContext, TextureContext) {
    let display_colormap_bytes = include_bytes!("../../assets/magma.png");
    let export_colormap_bytes = include_bytes!("../../assets/export_colormap.png");

    (
        generate_colormap_texture(device, queue, display_colormap_bytes),
        generate_colormap_texture(device, queue, export_colormap_bytes),
    )
}

/// Reads the passed bytes into a texture that can be bound to the colormap render pass
fn generate_colormap_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    colormap_bytes: &[u8],
) -> TextureContext {
    // Convert the passed bytes to an Image buffer
    let colormap_image = image::load_from_memory(colormap_bytes)
        .expect("ERROR: Failed to generate image from colormap_bytes");
    let colormap_rgba = colormap_image.to_rgba8();

    let dimensions = colormap_image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("colormap texture"),
        view_formats: &[],
    });

    // Fill in the texture with the data from the .png we read above
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &colormap_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(1),
        },
        texture_size,
    );

    // Set up a bind group for the texture
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D1,
                multisampled: false,
            },
            count: None,
        }],
        label: Some("colormap_bind_group_layout"),
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
        }],
        label: Some("colormap_bind_group"),
    });

    TextureContext {
        texture,
        bind_group_layout,
        bind_group,
    }
}

/// A texture capable of being copied to a buffer, we render the blend texture onto this texture when
///     calculating the max weight
pub fn generate_copy_texture(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::Texture {
    // Buffer size must be a multiple of 256 to map to CPU
    let tex_width = ceil((size.width as f64) / 256.0) as u32 * 256;

    let texture_size = wgpu::Extent3d {
        width: tex_width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("Max Weight Texture"),
        view_formats: &[],
    })
}

pub fn generate_export_texture(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> TextureContext {
    let export_texture_size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    // Create a 2D R16Float texture with appropriate usages
    let export_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: export_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: Some("export texture"),
        view_formats: &[],
    });

    // Set up a sampler for the texture we just created
    let export_texture_view = export_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let export_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Set up the bind group for the above texture and sampler
    let export_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
            label: Some("export_bind_group_layout"),
        });

    let export_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &export_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&export_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&export_sampler),
            },
        ],
        label: Some("export_bind_group"),
    });

    TextureContext {
        texture: export_texture,
        bind_group_layout: export_bind_group_layout,
        bind_group: export_bind_group,
    }
}

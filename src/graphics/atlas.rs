//! Pack textures into a single atlas to be uploaded to the GPU.

use std::borrow::Cow;

use glamour::{Rect, Size2, Vector2};
use packr2::{Packer, PackerConfig, Rectf, SkylinePacker};

use super::{
    state::PREFERRED_TEXTURE_FORMAT, texture::Texture, uniform::UniformArrayState, TextureRef,
};

/// Virtual packed texture size in pixels for both width and height.
const ATLAS_TEXTURE_SIZE: u32 = 4096;

/// A single packed atlas.
///
/// When the atlas is full another texture layer should be added.
pub(crate) struct Atlas {
    /// GPU reference.
    pub(crate) texture: wgpu::Texture,
    /// GPU bind group.
    pub(crate) bind_group: wgpu::BindGroup,
    /// GPU bind group layout.
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    /// GPU uniform buffer holding all atlassed texture rectangles.
    pub(crate) rects: UniformArrayState<Rect<f32>>,
    /// Packer algorithm used.
    packer: SkylinePacker,
}

impl Atlas {
    /// Create and upload the empty atlas to the GPU.
    pub(super) fn new(device: &wgpu::Device) -> Self {
        // Setup the texture packer state
        let packer = SkylinePacker::new(PackerConfig {
            max_width: ATLAS_TEXTURE_SIZE,
            max_height: ATLAS_TEXTURE_SIZE,
            allow_flipping: false,
        });

        // Create the texture on the GPU
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&Cow::Borrowed("Texture Atlas")),
            size: wgpu::Extent3d {
                width: ATLAS_TEXTURE_SIZE,
                height: ATLAS_TEXTURE_SIZE,
                // TODO: support multiple layers
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            // Texture is 2D
            dimension: wgpu::TextureDimension::D2,
            // Use sRGB format
            format: PREFERRED_TEXTURE_FORMAT,
            // We want to use this texture in shaders and we want to copy data to it
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            // We only need a single format
            view_formats: &[PREFERRED_TEXTURE_FORMAT],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Atlas Bind Group Layout"),
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
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create the buffer for the sizes
        let rects = UniformArrayState::new(device, 1024);

        Self {
            packer,
            texture,
            rects,
            bind_group,
            bind_group_layout,
        }
    }

    /// Add a texture to the atlas.
    ///
    /// # Returns
    ///
    /// - An unique identification number for the texture to be passed along with the vertices.
    pub(crate) fn add<T>(&mut self, mut texture: T, queue: &wgpu::Queue) -> TextureRef
    where
        T: Texture,
    {
        // Pack the rectangle
        let size = texture.size();
        let Rectf { x, y, w, h, .. } = self
            .packer
            .insert(size.width, size.height)
            .expect("New texture could not be packed, not enough space");

        log::debug!("Added texture to atlas at ({x}x{y}:{w}x{h})");

        assert_eq!(size.width, w);
        assert_eq!(size.height, h);

        // Write the sub-texture to the atlas location
        queue.write_texture(
            // Where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            // Actual pixel data
            &texture.to_rgba_image(),
            // Layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            // Texture size
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        // Push the newly packed dimensions to the uniform buffer, returning the reference to it
        let uniform_index = self.rects.push(
            &Rect::new(
                Vector2::new(x as f32, y as f32),
                Size2::new(w as f32, h as f32),
            ),
            queue,
        );

        uniform_index as TextureRef
    }
}

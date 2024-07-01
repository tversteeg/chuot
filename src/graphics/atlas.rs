//! Pack textures into a single atlas to be uploaded to the GPU.

use std::borrow::Cow;

use chuot_packer::Packer;

use super::{PREFERRED_TEXTURE_FORMAT, uniform::UniformArrayState};

/// Virtual packed texture size in pixels for both width and height.
pub(crate) const ATLAS_TEXTURE_SIZE: u32 = 4096;

/// Index into the atlas rectangles.
pub(crate) type TextureRef = u16;

/// A static packed atlas at compile time by the proc macro.
///
/// Will be unpacked and uploaded to the GPU once at the beginning of the game.
pub struct Atlas {
    /// GPU reference.
    pub(crate) texture: wgpu::Texture,
    /// GPU bind group.
    pub(crate) bind_group: wgpu::BindGroup,
    /// GPU bind group layout.
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    /// GPU uniform buffer holding all atlassed texture rectangles.
    pub(crate) rects: UniformArrayState<[f32; 4]>,
    /// Packer algorithm used.
    packer: Packer,
}

impl Atlas {
    /// Create and upload the atlas to the GPU.
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        // Create the texture on the GPU
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&Cow::Borrowed("Static Texture Atlas")),
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
            label: Some("Static Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Static Texture Atlas Bind Group Layout"),
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
            label: Some("Static Texture Atlas Bind Group"),
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

        // Setup a new atlas packer
        let packer = Packer::new((
            const { ATLAS_TEXTURE_SIZE as u16 },
            const { ATLAS_TEXTURE_SIZE as u16 },
        ));

        // Create and upload the uniforms
        let rects = UniformArrayState::new(device);

        Self {
            texture,
            bind_group,
            bind_group_layout,
            rects,
            packer,
        }
    }

    /// Add a texture to the atlas.
    ///
    /// # Returns
    ///
    /// - An unique identification number for the texture to be passed along with the vertices.
    pub(crate) fn add_texture(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) -> TextureRef {
        // Pack the rectangle
        let (x, y) = self
            .packer
            .insert((width as u16, height as u16))
            .expect("New texture could not be packed, not enough space");
        let x = x as u32;
        let y = y as u32;

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
            bytemuck::cast_slice(pixels),
            // Layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            // Texture size
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Push the newly packed dimensions to the uniform buffer, returning the reference to it
        let uniform_index = self
            .rects
            .push(&[x as f32, y as f32, width as f32, height as f32], queue);

        uniform_index as TextureRef
    }

    /// Update a region of pixels of the texture in the atlas.
    pub(crate) fn update_pixels(
        &self,
        texture_ref: TextureRef,
        (mut x, mut y, width, height): (f32, f32, f32, f32),
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) {
        // Get the region in the atlas for the already pushed sprite
        let [sprite_region_x, sprite_region_y, ..] = self.rects[texture_ref as usize];

        // Offset the sub rectangle to atlas space
        x += sprite_region_x;
        y += sprite_region_y;

        // Convert to u32 with proper rounding
        self.update_pixels_raw_offset(
            (
                x.round() as u32,
                y.round() as u32,
                width.round() as u32,
                height.round() as u32,
            ),
            pixels,
            queue,
        );
    }

    /// Update a region of pixels of the texture in the atlas.
    pub(crate) fn update_pixels_raw_offset(
        &self,
        (x, y, width, height): (u32, u32, u32, u32),
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) {
        // Write the new texture section to the GPU
        queue.write_texture(
            // Where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            // Actual pixel data
            bytemuck::cast_slice(pixels),
            // Layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            // Texture size
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

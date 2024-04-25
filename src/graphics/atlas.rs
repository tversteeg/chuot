//! Pack textures into a single atlas to be uploaded to the GPU.

use std::borrow::Cow;

use chuot_packer::Packer;
use glamour::{Point2, Rect, Size2, Vector2};

use super::{gpu::Gpu, state::PREFERRED_TEXTURE_FORMAT, uniform::UniformArrayState};

/// Virtual packed texture size in pixels for both width and height.
const ATLAS_TEXTURE_SIZE: u32 = 4096;

/// Index into the atlas rectangles.
pub(crate) type AtlasRef = u16;

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
    pub(crate) rects: UniformArrayState<Rect<f32>>,
    /// Packer algorithm used.
    packer: Packer,
}

impl Atlas {
    /// Create and upload the atlas to the GPU.
    pub(crate) fn new(texture_rects: Vec<Rect<f32>>, gpu: &Gpu) -> Self {
        // Create the texture on the GPU
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
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

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Static Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        // Create and upload the uniforms
        let rects = UniformArrayState::from_vec(texture_rects, &gpu.device);

        // Setup a new atlas packer
        let packer = Packer::new(Size2::splat(ATLAS_TEXTURE_SIZE as u16));

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
        size: Size2<u32>,
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) -> AtlasRef {
        // Pack the rectangle
        let Point2 { x, y } = self
            .packer
            .insert(Size2::new(size.width as u16, size.height as u16))
            .expect("New texture could not be packed, not enough space");
        let x = x as u32;
        let y = y as u32;

        log::debug!(
            "Added texture to atlas at ({x},{y}) with size {}x{}",
            size.width,
            size.height
        );

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
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            // Texture size
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );

        // Push the newly packed dimensions to the uniform buffer, returning the reference to it
        let uniform_index = self.rects.push(
            &Rect::new(
                Vector2::new(x as f32, y as f32),
                Size2::new(size.width as f32, size.height as f32),
            ),
            queue,
        );

        uniform_index as AtlasRef
    }

    /// Update a region of pixels of the texture in the atlas.
    pub(crate) fn update_pixels(
        &self,
        texture_ref: u16,
        mut sub_rectangle: Rect,
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) {
        // Get the region in the atlas for the already pushed sprite
        let sprite_region = self.rects[texture_ref as usize];

        // Offset the sub rectangle to atlas space
        sub_rectangle.origin += sprite_region.origin.to_vector();

        // Convert to u32 with proper rounding
        let region = Rect {
            origin: Point2::new(
                sub_rectangle.origin.x.round() as u32,
                sub_rectangle.origin.y.round() as u32,
            ),
            size: Size2::new(
                sub_rectangle.width().round() as u32,
                sub_rectangle.height().round() as u32,
            ),
        };

        self.update_pixels_raw_offset(region, pixels, queue);
    }

    /// Update a region of pixels of the texture in the atlas.
    pub(crate) fn update_pixels_raw_offset(
        &self,
        target_region: Rect<u32>,
        pixels: &[u32],
        queue: &wgpu::Queue,
    ) {
        // Write the new texture section to the GPU
        queue.write_texture(
            // Where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: target_region.origin.x,
                    y: target_region.origin.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            // Actual pixel data
            bytemuck::cast_slice(pixels),
            // Layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * target_region.width()),
                rows_per_image: Some(target_region.height()),
            },
            // Texture size
            wgpu::Extent3d {
                width: target_region.width(),
                height: target_region.height(),
                depth_or_array_layers: 1,
            },
        );
    }
}

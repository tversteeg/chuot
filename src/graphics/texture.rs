//! Allow types to expose a texture to the GPU.

use std::borrow::Cow;

use vek::Extent2;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor,
    Queue, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, Texture as GpuTexture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

/// Allow something to upload a texture to the GPU.
pub trait Texture {
    /// Dimensions of the texture.
    fn size(&self) -> Extent2<u32>;

    /// Raw pixels of the texture.
    ///
    /// Must be formatted as ARGB.
    fn pixels(&self) -> &[u32];

    /// Get the internal texture data.
    fn state(&self) -> Option<&TextureState>;

    /// Set the internal texture data.
    fn set_state(&mut self, state: TextureState);

    /// Upload the texture to the GPU.
    fn upload(&mut self, device: &Device) {
        let size = self.size();

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&Cow::Borrowed("Diffuse Texture")),
            size: Extent3d {
                width: size.w,
                height: size.h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            // Texture is 2D
            dimension: TextureDimension::D2,
            // Use sRGB format
            format: TextureFormat::Rgba8UnormSrgb,
            // We want to use this texture in shaders and we want to copy data to it
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            // We only need a single format
            view_formats: &[],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Diffuse Texture Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Diffuse Texture Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Store the state in the type implementing this
        self.set_state(TextureState {
            bind_group,
            texture,
            texture_view,
        });
    }

    /// Write data to the texture.
    fn write(&self, queue: &Queue, texture: &GpuTexture) {
        let size = self.size();

        queue.write_texture(
            // Where to copy the pixel data
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            // Actual pixel data
            bytemuck::cast_slice(self.pixels()),
            // Layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.w),
                rows_per_image: Some(size.h),
            },
            // Texture size
            Extent3d {
                width: size.w,
                height: size.h,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Simple texture state holding bind group and texture reference.
#[derive(Debug)]
pub struct TextureState {
    /// Bind group.
    pub(crate) bind_group: BindGroup,
    /// GPU texture reference.
    pub(crate) texture: GpuTexture,
    /// GPU texture view.
    pub(crate) texture_view: TextureView,
}

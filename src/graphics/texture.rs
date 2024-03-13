//! Allow types to expose a texture to the GPU.

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use assets_manager::SharedString;
use vek::Extent2;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor,
    Queue, RenderPass, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, Texture as GpuTexture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

use crate::assets::image::Image;

use super::MainRenderState;

/// Reference to an uploaded texture to render.
pub type TextureRef = SharedString;

/// Uploaded textures.
static UPLOADED_TEXTURES: OnceLock<Arc<Mutex<HashMap<TextureRef, UploadedTextureState>>>> =
    OnceLock::new();

/// Textures that are waiting to be uploaded to the GPU.
static PENDING_TEXTURES: OnceLock<Arc<Mutex<HashMap<TextureRef, PendingTextureState>>>> =
    OnceLock::new();

/// Allow something to upload a texture to the GPU.
pub trait Texture {
    /// Dimensions of the texture.
    fn size(&self) -> Extent2<u32>;

    /// Raw pixels of the texture.
    ///
    /// Must be formatted as ARGB.
    fn pixels(&self) -> &[u32];

    /// Upload the texture to the GPU.
    fn upload(
        &self,
        device: &Device,
        queue: &Queue,
        diffuse_texture_bind_group_layout: &BindGroupLayout,
    ) -> UploadedTextureState {
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

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse Texture Bind Group"),
            layout: diffuse_texture_bind_group_layout,
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

        // Store the initial texture data
        self.write(&queue, &texture);

        // Store the result
        UploadedTextureState {
            bind_group,
            texture,
            texture_view,
        }
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

/// Texture state for textures that have been uploaded to the GPU holding bind group and texture reference.
#[derive(Debug)]
pub struct UploadedTextureState {
    /// Bind group.
    pub(crate) bind_group: BindGroup,
    /// GPU texture reference.
    pub(crate) texture: GpuTexture,
    /// GPU texture view.
    pub(crate) texture_view: TextureView,
}

/// Texture state that still needs to be uploaded to the GPU.
pub struct PendingTextureState(Box<dyn Texture + Send>);

/// Store an texture waiting to be uploaded.
pub(crate) fn store<T>(id: SharedString, texture: T) -> TextureRef
where
    T: Texture + Send,
{
    // Get a reference to the pending textures map
    let mut pending_textures = PENDING_TEXTURES
        // If it doesn't exist yet create a new one
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock()
        .expect("Error locking mutex");

    // Insert the texture on the heap, so multiple types can be put in the same map
    pending_textures.insert(id.clone(), PendingTextureState(Box::new(texture)));

    id
}

/// Set the bind group to a render pass for an uploaded texture.
pub(crate) fn set_bind_group(reference: &TextureRef, render_pass: &mut RenderPass, index: u32) {
    // Get a reference to the uploaded textures map
    let uploaded_textures = UPLOADED_TEXTURES
        // If it doesn't exist yet create a new one
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock()
        .expect("Error locking mutex");

    let texture_state = uploaded_textures
        .get(reference)
        .expect("Texture not uploaded yet");

    render_pass.set_bind_group(index, &texture_state.bind_group, &[]);
}

/// Upload all missing textures to the GPU.
pub(crate) fn upload(
    device: &Device,
    queue: &Queue,
    diffuse_texture_bind_group_layout: &BindGroupLayout,
) {
    // Get a reference to the pending textures map
    let mut pending_textures = PENDING_TEXTURES
        // If it doesn't exist yet create a new one
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock()
        .expect("Error locking mutex");

    // Get a reference to the uploaded textures map
    let mut uploaded_textures = UPLOADED_TEXTURES
        // If it doesn't exist yet create a new one
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .lock()
        .expect("Error locking mutex");

    pending_textures
        .drain()
        .for_each(|(texture_ref, pending_texture)| {
            // Upload each texture and insert it into the uploaded map
            uploaded_textures.insert(
                texture_ref,
                pending_texture
                    .0
                    .upload(device, queue, diffuse_texture_bind_group_layout),
            );
        });
}

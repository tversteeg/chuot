//! Allow types to expose a texture to the GPU.

use std::{
    borrow::Cow,
    sync::{Arc, Mutex, OnceLock},
};

use assets_manager::SharedString;
use bytemuck::{Pod, Zeroable};
use glamour::Size2;
use hashbrown::HashMap;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, SamplerBindingType,
    SamplerDescriptor, ShaderStages, Texture as GpuTexture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension,
};

/// Reference to an uploaded texture to render.
pub type TextureRef = SharedString;

/// Textures that are waiting to be uploaded to the GPU.
pub(super) static PENDING_TEXTURES: OnceLock<Arc<Mutex<HashMap<TextureRef, PendingTextureState>>>> =
    OnceLock::new();

/// Allow something to upload a texture to the GPU.
pub trait Texture {
    /// Dimensions of the texture.
    fn size(&self) -> Size2<u32>;

    /// Image representation we can upload to the GPU.
    fn to_rgba_image(&mut self) -> Vec<u8>;
}

/// Texture size for the uniform.
#[repr(C, align(16))]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub(super) struct TextureInfo {
    /// Size of the texture in pixels.
    pub(super) size: [f32; 2],
    /// Padding for alignment.
    pub(super) _padding: [f32; 2],
}

/// Texture state for textures that have been uploaded to the GPU holding bind group and texture reference.
#[derive(Debug)]
pub(super) struct UploadedTextureState {
    /// Bind group.
    pub(super) bind_group: BindGroup,
    /// GPU texture reference.
    pub(super) texture: GpuTexture,
}

/// Texture state that still needs to be uploaded to the GPU.
pub(super) struct PendingTextureState(Box<dyn Texture + Send>);

impl PendingTextureState {
    /// Upload the texture.
    pub(super) fn upload(
        &self,
        device: &Device,
        diffuse_texture_bind_group_layout: &BindGroupLayout,
    ) -> UploadedTextureState {
        let size = self.0.size();

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&Cow::Borrowed("Diffuse Texture")),
            size: Extent3d {
                width: size.width,
                height: size.height,
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

        // Store the result
        UploadedTextureState {
            bind_group,
            texture,
        }
    }

    /// Write the texture data to the GPU.
    pub(super) fn write(mut self, queue: &Queue, uploaded_texture_state: &UploadedTextureState) {
        let size = self.0.size();

        queue.write_texture(
            // Where to copy the pixel data
            ImageCopyTexture {
                texture: &uploaded_texture_state.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            // Actual pixel data
            &self.0.to_rgba_image(),
            // Layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            // Texture size
            Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Store an texture waiting to be uploaded.
pub(crate) fn upload<T>(id: SharedString, texture: T) -> TextureRef
where
    T: Texture + Send + 'static,
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

/// Create the bind group layout.
pub(super) fn create_bind_group_layout(device: &Device, name: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(name),
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
    })
}

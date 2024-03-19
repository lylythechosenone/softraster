use std::alloc::Layout;

use wgpu_hal::TextureDescriptor;
use wgpu_types::{Extent3d, TextureDimension, TextureFormat};

pub(super) struct Unsupported;

#[derive(Debug)]
pub struct Texture {
    pub size: Extent3d,
    pub dimension: TextureDimension,
    pub pixel_size: usize,
    pub data: Box<[u8]>,
}
impl Texture {
    pub(super) unsafe fn new(desc: &TextureDescriptor) -> Result<Self, Unsupported> {
        let pixel_width = match desc.format {
            TextureFormat::R8Unorm
            | TextureFormat::R8Snorm
            | TextureFormat::R8Uint
            | TextureFormat::R8Sint
            | TextureFormat::Stencil8 => 1,
            TextureFormat::R16Uint
            | TextureFormat::R16Sint
            | TextureFormat::R16Unorm
            | TextureFormat::R16Snorm
            | TextureFormat::R16Float
            | TextureFormat::Rg8Unorm
            | TextureFormat::Rg8Snorm
            | TextureFormat::Rg8Uint
            | TextureFormat::Rg8Sint
            | TextureFormat::Depth16Unorm => 2,
            TextureFormat::R32Uint
            | TextureFormat::R32Sint
            | TextureFormat::R32Float
            | TextureFormat::Rg16Uint
            | TextureFormat::Rg16Sint
            | TextureFormat::Rg16Unorm
            | TextureFormat::Rg16Snorm
            | TextureFormat::Rg16Float
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Snorm
            | TextureFormat::Rgba8Uint
            | TextureFormat::Rgba8Sint
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Rgb9e5Ufloat
            | TextureFormat::Rgb10a2Uint
            | TextureFormat::Rgb10a2Unorm
            | TextureFormat::Rg11b10Float
            | TextureFormat::Depth24Plus
            | TextureFormat::Depth24PlusStencil8
            | TextureFormat::Depth32Float => 4,
            TextureFormat::Rg32Uint
            | TextureFormat::Rg32Sint
            | TextureFormat::Rg32Float
            | TextureFormat::Rgba16Uint
            | TextureFormat::Rgba16Sint
            | TextureFormat::Rgba16Unorm
            | TextureFormat::Rgba16Snorm
            | TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Uint | TextureFormat::Rgba32Sint | TextureFormat::Rgba32Float => {
                16
            }
            _ => return Err(Unsupported),
        };

        Ok(Self {
            size: desc.size,
            dimension: if desc.size.depth_or_array_layers > 1 {
                TextureDimension::D3
            } else {
                desc.dimension
            },
            pixel_size: pixel_width,
            data: super::alloc(
                Layout::from_size_align(
                    pixel_width
                        * desc.size.width as usize
                        * desc.size.height as usize
                        * desc.size.depth_or_array_layers as usize,
                    pixel_width,
                )
                .unwrap(),
            ),
        })
    }
}

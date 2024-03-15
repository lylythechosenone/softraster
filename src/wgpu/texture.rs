use std::mem::MaybeUninit;

use wgpu_types::{Extent3d, TextureDimension, TextureFormat};

#[derive(Debug)]
pub struct Texture {
    pub size: Extent3d,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub data: Box<[MaybeUninit<u8>]>,
}

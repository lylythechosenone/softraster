#![allow(clippy::cast_possible_truncation)]

use std::{alloc::Layout, borrow::Borrow};

use self::{command_encoder::Command, texture::Texture};

pub mod adapter;
pub mod command_encoder;
pub mod device;
pub mod instance;
pub mod queue;
pub mod surface;
pub mod texture;

#[derive(Clone, Copy, Debug)]
pub struct Todo;
impl Borrow<<Api as wgpu_hal::Api>::Texture> for Todo {
    fn borrow(&self) -> &<Api as wgpu_hal::Api>::Texture {
        todo!()
    }
}

unsafe fn alloc(layout: Layout) -> Box<[u8]> {
    Box::from_raw(core::ptr::slice_from_raw_parts_mut(
        std::alloc::alloc_zeroed(layout).cast(),
        layout.size(),
    ))
}

#[derive(Clone, Copy, Debug)]
pub struct Api;
impl wgpu_hal::Api for Api {
    type Instance = instance::Instance;
    type Surface = surface::Surface;
    type Adapter = adapter::Adapter;
    type Device = device::Device;
    type Queue = queue::Queue;
    type CommandEncoder = command_encoder::CommandEncoder;
    type CommandBuffer = Vec<Command>;
    type Buffer = Box<[u8]>;
    type Texture = Texture;
    type SurfaceTexture = Todo;
    type TextureView = Todo;
    type Sampler = Todo;
    type QuerySet = Todo;
    type Fence = Todo;
    type BindGroupLayout = Todo;
    type BindGroup = Todo;
    type PipelineLayout = Todo;
    type ShaderModule = Todo;
    type RenderPipeline = Todo;
    type ComputePipeline = Todo;
    type AccelerationStructure = Todo;
}

use std::{borrow::Borrow, mem::MaybeUninit};

use self::texture::Texture;

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

#[derive(Clone, Copy, Debug)]
pub struct Api;
impl wgpu_hal::Api for Api {
    type Instance = instance::Instance;
    type Surface = surface::Surface;
    type Adapter = adapter::Adapter;
    type Device = device::Device;
    type Queue = queue::Queue;
    type CommandEncoder = command_encoder::CommandEncoder;
    type CommandBuffer = Todo;
    type Buffer = Box<[MaybeUninit<u8>]>;
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

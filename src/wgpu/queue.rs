use super::Api;

pub struct Queue;
impl wgpu_hal::Queue<Api> for Queue {
    unsafe fn submit(
        &self,
        command_buffers: &[&<Api as wgpu_hal::Api>::CommandBuffer],
        signal_fence: Option<(&mut <Api as wgpu_hal::Api>::Fence, wgpu_hal::FenceValue)>,
    ) -> Result<(), wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn present(
        &self,
        surface: &<Api as wgpu_hal::Api>::Surface,
        texture: <Api as wgpu_hal::Api>::SurfaceTexture,
    ) -> Result<(), wgpu_hal::SurfaceError> {
        todo!()
    }

    unsafe fn get_timestamp_period(&self) -> f32 {
        todo!()
    }
}

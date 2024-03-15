use super::Api;

pub struct Surface;
impl wgpu_hal::Surface<Api> for Surface {
    unsafe fn configure(
        &self,
        _device: &<Api as wgpu_hal::Api>::Device,
        _config: &wgpu_hal::SurfaceConfiguration,
    ) -> Result<(), wgpu_hal::SurfaceError> {
        todo!()
    }

    unsafe fn unconfigure(&self, _device: &<Api as wgpu_hal::Api>::Device) {
        todo!()
    }

    unsafe fn acquire_texture(
        &self,
        _timeout: Option<std::time::Duration>,
    ) -> Result<Option<wgpu_hal::AcquiredSurfaceTexture<Api>>, wgpu_hal::SurfaceError> {
        todo!()
    }

    unsafe fn discard_texture(&self, _texture: <Api as wgpu_hal::Api>::SurfaceTexture) {
        todo!()
    }
}

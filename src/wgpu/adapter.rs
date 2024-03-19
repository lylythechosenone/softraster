use super::Api;

pub struct Adapter;
impl wgpu_hal::Adapter<Api> for Adapter {
    unsafe fn open(
        &self,
        features: wgpu_types::Features,
        limits: &wgpu_types::Limits,
    ) -> Result<wgpu_hal::OpenDevice<Api>, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn texture_format_capabilities(
        &self,
        format: wgpu_types::TextureFormat,
    ) -> wgpu_hal::TextureFormatCapabilities {
        todo!()
    }

    unsafe fn surface_capabilities(
        &self,
        _surface: &<Api as wgpu_hal::Api>::Surface,
    ) -> Option<wgpu_hal::SurfaceCapabilities> {
        todo!()
    }

    unsafe fn get_presentation_timestamp(&self) -> wgpu_types::PresentationTimestamp {
        todo!()
    }
}

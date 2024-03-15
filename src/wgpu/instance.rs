use super::Api;

pub struct Instance {
    name: String,
    flags: wgpu_types::InstanceFlags,
}
impl wgpu_hal::Instance<Api> for Instance {
    unsafe fn init(desc: &wgpu_hal::InstanceDescriptor) -> Result<Self, wgpu_hal::InstanceError> {
        Ok(Self {
            name: desc.name.to_string(),
            flags: desc.flags,
        })
    }

    unsafe fn create_surface(
        &self,
        _display_handle: raw_window_handle::RawDisplayHandle,
        _window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<<super::Api as wgpu_hal::Api>::Surface, wgpu_hal::InstanceError> {
        todo!("surfaces (use only offscreen rendering for now)")
    }

    unsafe fn destroy_surface(&self, _surface: <super::Api as wgpu_hal::Api>::Surface) {}

    unsafe fn enumerate_adapters(&self) -> Vec<wgpu_hal::ExposedAdapter<super::Api>> {
        todo!()
    }
}

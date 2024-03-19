use std::sync::{Arc, Mutex};

use super::{command::Command, Api};

pub struct Queue {
    commands: Arc<Mutex<Vec<Command>>>,
}
impl wgpu_hal::Queue<Api> for Queue {
    unsafe fn submit(
        &self,
        command_buffers: &[&<Api as wgpu_hal::Api>::CommandBuffer],
        signal_fence: Option<(&mut <Api as wgpu_hal::Api>::Fence, wgpu_hal::FenceValue)>,
    ) -> Result<(), wgpu_hal::DeviceError> {
        let mut lock = self.commands.lock().unwrap();

        for buffer in command_buffers {
            lock.extend_from_slice(buffer);
        }

        if let Some((fence, value)) = signal_fence {
            lock.push(Command::Signal {
                fence: &*std::ptr::from_ref(fence),
                value,
            });
        }

        Ok(())
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

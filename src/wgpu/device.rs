use std::{alloc::Layout, ptr::NonNull};

use wgpu_hal::DeviceError;

use super::{fence::Fence, texture::Texture, Api};

pub struct Device;
impl wgpu_hal::Device<Api> for Device {
    unsafe fn exit(self, queue: <Api as wgpu_hal::Api>::Queue) {
        todo!()
    }

    unsafe fn create_buffer(
        &self,
        desc: &wgpu_hal::BufferDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::Buffer, wgpu_hal::DeviceError> {
        Ok(super::alloc(
            Layout::from_size_align(desc.size as usize, 32)
                .map_err(|_| DeviceError::ResourceCreationFailed)?,
        ))
    }

    unsafe fn destroy_buffer(&self, _buffer: <Api as wgpu_hal::Api>::Buffer) {}

    unsafe fn map_buffer(
        &self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        range: wgpu_hal::MemoryRange,
    ) -> Result<wgpu_hal::BufferMapping, wgpu_hal::DeviceError> {
        Ok(wgpu_hal::BufferMapping {
            ptr: NonNull::new_unchecked(
                buffer
                    .as_ptr()
                    .byte_add(range.start as usize)
                    .cast_mut()
                    .cast(),
            ),
            is_coherent: false,
        })
    }

    unsafe fn unmap_buffer(
        &self,
        _buffer: &<Api as wgpu_hal::Api>::Buffer,
    ) -> Result<(), wgpu_hal::DeviceError> {
        Ok(())
    }

    unsafe fn flush_mapped_ranges<I>(&self, _buffer: &<Api as wgpu_hal::Api>::Buffer, _ranges: I)
    where
        I: Iterator<Item = wgpu_hal::MemoryRange>,
    {
    }

    unsafe fn invalidate_mapped_ranges<I>(
        &self,
        _buffer: &<Api as wgpu_hal::Api>::Buffer,
        _ranges: I,
    ) where
        I: Iterator<Item = wgpu_hal::MemoryRange>,
    {
    }

    unsafe fn create_texture(
        &self,
        desc: &wgpu_hal::TextureDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::Texture, wgpu_hal::DeviceError> {
        Texture::new(desc).map_err(|_| DeviceError::ResourceCreationFailed)
    }

    unsafe fn destroy_texture(&self, _texture: <Api as wgpu_hal::Api>::Texture) {}

    unsafe fn create_texture_view(
        &self,
        texture: &<Api as wgpu_hal::Api>::Texture,
        desc: &wgpu_hal::TextureViewDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::TextureView, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_texture_view(&self, view: <Api as wgpu_hal::Api>::TextureView) {
        todo!()
    }

    unsafe fn create_sampler(
        &self,
        desc: &wgpu_hal::SamplerDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::Sampler, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_sampler(&self, sampler: <Api as wgpu_hal::Api>::Sampler) {
        todo!()
    }

    unsafe fn create_command_encoder(
        &self,
        desc: &wgpu_hal::CommandEncoderDescriptor<Api>,
    ) -> Result<<Api as wgpu_hal::Api>::CommandEncoder, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_command_encoder(&self, pool: <Api as wgpu_hal::Api>::CommandEncoder) {
        todo!()
    }

    unsafe fn create_bind_group_layout(
        &self,
        desc: &wgpu_hal::BindGroupLayoutDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::BindGroupLayout, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_bind_group_layout(&self, bg_layout: <Api as wgpu_hal::Api>::BindGroupLayout) {
        todo!()
    }

    unsafe fn create_pipeline_layout(
        &self,
        desc: &wgpu_hal::PipelineLayoutDescriptor<Api>,
    ) -> Result<<Api as wgpu_hal::Api>::PipelineLayout, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_pipeline_layout(
        &self,
        pipeline_layout: <Api as wgpu_hal::Api>::PipelineLayout,
    ) {
        todo!()
    }

    unsafe fn create_bind_group(
        &self,
        desc: &wgpu_hal::BindGroupDescriptor<Api>,
    ) -> Result<<Api as wgpu_hal::Api>::BindGroup, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_bind_group(&self, group: <Api as wgpu_hal::Api>::BindGroup) {
        todo!()
    }

    unsafe fn create_shader_module(
        &self,
        desc: &wgpu_hal::ShaderModuleDescriptor,
        shader: wgpu_hal::ShaderInput,
    ) -> Result<<Api as wgpu_hal::Api>::ShaderModule, wgpu_hal::ShaderError> {
        todo!()
    }

    unsafe fn destroy_shader_module(&self, module: <Api as wgpu_hal::Api>::ShaderModule) {
        todo!()
    }

    unsafe fn create_render_pipeline(
        &self,
        desc: &wgpu_hal::RenderPipelineDescriptor<Api>,
    ) -> Result<<Api as wgpu_hal::Api>::RenderPipeline, wgpu_hal::PipelineError> {
        todo!()
    }

    unsafe fn destroy_render_pipeline(&self, pipeline: <Api as wgpu_hal::Api>::RenderPipeline) {
        todo!()
    }

    unsafe fn create_compute_pipeline(
        &self,
        desc: &wgpu_hal::ComputePipelineDescriptor<Api>,
    ) -> Result<<Api as wgpu_hal::Api>::ComputePipeline, wgpu_hal::PipelineError> {
        todo!()
    }

    unsafe fn destroy_compute_pipeline(&self, pipeline: <Api as wgpu_hal::Api>::ComputePipeline) {
        todo!()
    }

    unsafe fn create_query_set(
        &self,
        desc: &wgpu_types::QuerySetDescriptor<wgpu_hal::Label>,
    ) -> Result<<Api as wgpu_hal::Api>::QuerySet, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn destroy_query_set(&self, set: <Api as wgpu_hal::Api>::QuerySet) {
        todo!()
    }

    unsafe fn create_fence(&self) -> Result<<Api as wgpu_hal::Api>::Fence, wgpu_hal::DeviceError> {
        Ok(Fence::new())
    }

    unsafe fn destroy_fence(&self, _fence: <Api as wgpu_hal::Api>::Fence) {}

    unsafe fn get_fence_value(
        &self,
        fence: &<Api as wgpu_hal::Api>::Fence,
    ) -> Result<wgpu_hal::FenceValue, wgpu_hal::DeviceError> {
        Ok(fence.get_value())
    }

    unsafe fn wait(
        &self,
        fence: &<Api as wgpu_hal::Api>::Fence,
        value: wgpu_hal::FenceValue,
        timeout_ms: u32,
    ) -> Result<bool, wgpu_hal::DeviceError> {
        Ok(fence.wait(value, timeout_ms))
    }

    unsafe fn start_capture(&self) -> bool {
        todo!()
    }

    unsafe fn stop_capture(&self) {
        todo!()
    }

    unsafe fn create_acceleration_structure(
        &self,
        desc: &wgpu_hal::AccelerationStructureDescriptor,
    ) -> Result<<Api as wgpu_hal::Api>::AccelerationStructure, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn get_acceleration_structure_build_sizes(
        &self,
        desc: &wgpu_hal::GetAccelerationStructureBuildSizesDescriptor<Api>,
    ) -> wgpu_hal::AccelerationStructureBuildSizes {
        todo!()
    }

    unsafe fn get_acceleration_structure_device_address(
        &self,
        acceleration_structure: &<Api as wgpu_hal::Api>::AccelerationStructure,
    ) -> wgpu_types::BufferAddress {
        todo!()
    }

    unsafe fn destroy_acceleration_structure(
        &self,
        acceleration_structure: <Api as wgpu_hal::Api>::AccelerationStructure,
    ) {
        todo!()
    }
}

use super::Api;

#[derive(Debug)]
pub struct CommandEncoder;
impl wgpu_hal::CommandEncoder<Api> for CommandEncoder {
    unsafe fn begin_encoding(
        &mut self,
        label: wgpu_hal::Label,
    ) -> Result<(), wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn discard_encoding(&mut self) {
        todo!()
    }

    unsafe fn end_encoding(
        &mut self,
    ) -> Result<<Api as wgpu_hal::Api>::CommandBuffer, wgpu_hal::DeviceError> {
        todo!()
    }

    unsafe fn reset_all<I>(&mut self, command_buffers: I)
    where
        I: Iterator<Item = <Api as wgpu_hal::Api>::CommandBuffer>,
    {
        todo!()
    }

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = wgpu_hal::BufferBarrier<'a, Api>>,
    {
        todo!()
    }

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = wgpu_hal::TextureBarrier<'a, Api>>,
    {
        todo!()
    }

    unsafe fn clear_buffer(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        range: wgpu_hal::MemoryRange,
    ) {
        todo!()
    }

    unsafe fn copy_buffer_to_buffer<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Buffer,
        dst: &<Api as wgpu_hal::Api>::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferCopy>,
    {
        todo!()
    }

    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Texture,
        src_usage: wgpu_hal::TextureUses,
        dst: &<Api as wgpu_hal::Api>::Texture,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::TextureCopy>,
    {
        todo!()
    }

    unsafe fn copy_buffer_to_texture<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Buffer,
        dst: &<Api as wgpu_hal::Api>::Texture,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferTextureCopy>,
    {
        todo!()
    }

    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Texture,
        src_usage: wgpu_hal::TextureUses,
        dst: &<Api as wgpu_hal::Api>::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferTextureCopy>,
    {
        todo!()
    }

    unsafe fn set_bind_group(
        &mut self,
        layout: &<Api as wgpu_hal::Api>::PipelineLayout,
        index: u32,
        group: &<Api as wgpu_hal::Api>::BindGroup,
        dynamic_offsets: &[wgpu_types::DynamicOffset],
    ) {
        todo!()
    }

    unsafe fn set_push_constants(
        &mut self,
        layout: &<Api as wgpu_hal::Api>::PipelineLayout,
        stages: wgpu_types::ShaderStages,
        offset_bytes: u32,
        data: &[u32],
    ) {
        todo!()
    }

    unsafe fn insert_debug_marker(&mut self, label: &str) {
        todo!()
    }

    unsafe fn begin_debug_marker(&mut self, group_label: &str) {
        todo!()
    }

    unsafe fn end_debug_marker(&mut self) {
        todo!()
    }

    unsafe fn begin_query(&mut self, set: &<Api as wgpu_hal::Api>::QuerySet, index: u32) {
        todo!()
    }

    unsafe fn end_query(&mut self, set: &<Api as wgpu_hal::Api>::QuerySet, index: u32) {
        todo!()
    }

    unsafe fn write_timestamp(&mut self, set: &<Api as wgpu_hal::Api>::QuerySet, index: u32) {
        todo!()
    }

    unsafe fn reset_queries(
        &mut self,
        set: &<Api as wgpu_hal::Api>::QuerySet,
        range: std::ops::Range<u32>,
    ) {
        todo!()
    }

    unsafe fn copy_query_results(
        &mut self,
        set: &<Api as wgpu_hal::Api>::QuerySet,
        range: std::ops::Range<u32>,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
        stride: wgpu_types::BufferSize,
    ) {
        todo!()
    }

    unsafe fn begin_render_pass(&mut self, desc: &wgpu_hal::RenderPassDescriptor<Api>) {
        todo!()
    }

    unsafe fn end_render_pass(&mut self) {
        todo!()
    }

    unsafe fn set_render_pipeline(&mut self, pipeline: &<Api as wgpu_hal::Api>::RenderPipeline) {
        todo!()
    }

    unsafe fn set_index_buffer(
        &mut self,
        binding: wgpu_hal::BufferBinding<'_, Api>,
        format: wgpu_types::IndexFormat,
    ) {
        todo!()
    }

    unsafe fn set_vertex_buffer(&mut self, index: u32, binding: wgpu_hal::BufferBinding<'_, Api>) {
        todo!()
    }

    unsafe fn set_viewport(
        &mut self,
        rect: &wgpu_hal::Rect<f32>,
        depth_range: std::ops::Range<f32>,
    ) {
        todo!()
    }

    unsafe fn set_scissor_rect(&mut self, rect: &wgpu_hal::Rect<u32>) {
        todo!()
    }

    unsafe fn set_stencil_reference(&mut self, value: u32) {
        todo!()
    }

    unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {
        todo!()
    }

    unsafe fn draw(
        &mut self,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
    ) {
        todo!()
    }

    unsafe fn draw_indexed(
        &mut self,
        first_index: u32,
        index_count: u32,
        base_vertex: i32,
        first_instance: u32,
        instance_count: u32,
    ) {
        todo!()
    }

    unsafe fn draw_indirect(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
        draw_count: u32,
    ) {
        todo!()
    }

    unsafe fn draw_indexed_indirect(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
        draw_count: u32,
    ) {
        todo!()
    }

    unsafe fn draw_indirect_count(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
        count_buffer: &<Api as wgpu_hal::Api>::Buffer,
        count_offset: wgpu_types::BufferAddress,
        max_count: u32,
    ) {
        todo!()
    }

    unsafe fn draw_indexed_indirect_count(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
        count_buffer: &<Api as wgpu_hal::Api>::Buffer,
        count_offset: wgpu_types::BufferAddress,
        max_count: u32,
    ) {
        todo!()
    }

    unsafe fn begin_compute_pass(&mut self, desc: &wgpu_hal::ComputePassDescriptor<Api>) {
        todo!()
    }

    unsafe fn end_compute_pass(&mut self) {
        todo!()
    }

    unsafe fn set_compute_pipeline(&mut self, pipeline: &<Api as wgpu_hal::Api>::ComputePipeline) {
        todo!()
    }

    unsafe fn dispatch(&mut self, count: [u32; 3]) {
        todo!()
    }

    unsafe fn dispatch_indirect(
        &mut self,
        buffer: &<Api as wgpu_hal::Api>::Buffer,
        offset: wgpu_types::BufferAddress,
    ) {
        todo!()
    }

    unsafe fn build_acceleration_structures<'a, T>(&mut self, descriptor_count: u32, descriptors: T)
    where
        Api: 'a,
        T: IntoIterator<Item = wgpu_hal::BuildAccelerationStructureDescriptor<'a, Api>>,
    {
        todo!()
    }

    unsafe fn place_acceleration_structure_barrier(
        &mut self,
        barrier: wgpu_hal::AccelerationStructureBarrier,
    ) {
        todo!()
    }
}

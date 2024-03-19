use wgpu_hal::BufferCopy;
use wgpu_types::TextureDimension;

use super::Api;

#[derive(Debug)]
pub enum Command {
    Clear {
        buffer: *mut [u8],
    },
    Copy {
        src: *mut [u8],
        dst: *mut u8,
    },
    Copy2 {
        src: *mut u8,
        dst: *mut u8,
        src_pitch: usize,
        dst_pitch: usize,
        rows: usize,
        columns: usize,
    },
    Copy3 {
        src: *mut u8,
        dst: *mut u8,
        src_pitch: usize,
        dst_pitch: usize,
        src_depth_pitch: usize,
        dst_depth_pitch: usize,
        rows: usize,
        columns: usize,
        depth: usize,
    },
}
unsafe impl Send for Command {}
unsafe impl Sync for Command {}

#[derive(Debug)]
pub struct CommandEncoder {
    commands: Vec<Command>,
}
impl wgpu_hal::CommandEncoder<Api> for CommandEncoder {
    unsafe fn begin_encoding(
        &mut self,
        _label: wgpu_hal::Label,
    ) -> Result<(), wgpu_hal::DeviceError> {
        Ok(())
    }

    unsafe fn discard_encoding(&mut self) {
        self.commands.clear();
    }

    unsafe fn end_encoding(
        &mut self,
    ) -> Result<<Api as wgpu_hal::Api>::CommandBuffer, wgpu_hal::DeviceError> {
        let mut commands = Vec::new();
        core::mem::swap(&mut self.commands, &mut commands);
        Ok(commands)
    }

    unsafe fn reset_all<I>(&mut self, command_buffers: I)
    where
        I: Iterator<Item = <Api as wgpu_hal::Api>::CommandBuffer>,
    {
        for mut buffer in command_buffers {
            if buffer.capacity() > self.commands.capacity() {
                buffer.clear();
                buffer.append(&mut self.commands);
                self.commands = buffer;
            }
        }
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
        self.commands.push(Command::Clear {
            buffer: core::ptr::slice_from_raw_parts_mut(
                buffer.as_ptr().add(range.start as usize).cast_mut().cast(),
                (range.end - range.start) as usize,
            ),
        });
    }

    unsafe fn copy_buffer_to_buffer<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Buffer,
        dst: &<Api as wgpu_hal::Api>::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferCopy>,
    {
        for BufferCopy {
            src_offset,
            dst_offset,
            size,
        } in regions
        {
            self.commands.push(Command::Copy {
                src: core::ptr::slice_from_raw_parts_mut(
                    src.as_ptr().add(src_offset as usize).cast_mut().cast(),
                    size.get() as usize,
                ),
                dst: dst.as_ptr().add(dst_offset as usize).cast_mut().cast(),
            });
        }
    }

    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Texture,
        _src_usage: wgpu_hal::TextureUses,
        dst: &<Api as wgpu_hal::Api>::Texture,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::TextureCopy>,
    {
        match src.dimension {
            TextureDimension::D1 => {
                for region in regions {
                    self.commands.push(Command::Copy {
                        src: core::slice::from_raw_parts_mut(
                            src.data
                                .as_ptr()
                                .add(region.src_base.origin.x as usize * src.pixel_size)
                                .cast_mut()
                                .cast(),
                            region.size.width as usize * src.pixel_size,
                        ),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(region.dst_base.origin.x as usize * src.pixel_size)
                            .cast_mut()
                            .cast(),
                    });
                }
            }
            TextureDimension::D2 => {
                let src_pitch = src.pixel_size * src.size.width as usize;
                let dst_pitch = src.pixel_size * dst.size.width as usize;
                for region in regions {
                    self.commands.push(Command::Copy2 {
                        src: src
                            .data
                            .as_ptr()
                            .add(
                                region.src_base.origin.x as usize
                                    + region.src_base.origin.y as usize * src_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(
                                region.dst_base.origin.x as usize
                                    + region.dst_base.origin.y as usize * dst_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        src_pitch,
                        dst_pitch,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                    });
                }
            }
            TextureDimension::D3 => {
                let src_pitch = src.pixel_size * src.size.width as usize;
                let dst_pitch = src.pixel_size * dst.size.width as usize;
                let src_depth_pitch =
                    src.pixel_size * src.size.width as usize * src.size.height as usize;
                let dst_depth_pitch =
                    src.pixel_size * dst.size.width as usize * dst.size.height as usize;
                for region in regions {
                    self.commands.push(Command::Copy3 {
                        src: src
                            .data
                            .as_ptr()
                            .add(
                                region.src_base.origin.x as usize
                                    + region.src_base.origin.y as usize * src_pitch
                                    + region.src_base.origin.z.max(region.src_base.array_layer)
                                        as usize
                                        * src_depth_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(
                                region.dst_base.origin.x as usize
                                    + region.dst_base.origin.y as usize * dst_pitch
                                    + region.dst_base.origin.z.max(region.dst_base.array_layer)
                                        as usize
                                        * dst_depth_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        src_pitch,
                        dst_pitch,
                        src_depth_pitch,
                        dst_depth_pitch,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                        depth: region.size.depth as usize,
                    });
                }
            }
        }
    }

    unsafe fn copy_buffer_to_texture<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Buffer,
        dst: &<Api as wgpu_hal::Api>::Texture,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferTextureCopy>,
    {
        match dst.dimension {
            TextureDimension::D1 => {
                for region in regions {
                    self.commands.push(Command::Copy {
                        src: core::ptr::slice_from_raw_parts_mut(
                            src.as_ptr()
                                .add(region.buffer_layout.offset as usize)
                                .cast_mut()
                                .cast(),
                            region.size.width as usize,
                        ),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(region.texture_base.origin.x as usize)
                            .cast_mut()
                            .cast(),
                    });
                }
            }
            TextureDimension::D2 => {
                let dst_pitch = dst.pixel_size * dst.size.width as usize;
                for region in regions {
                    self.commands.push(Command::Copy2 {
                        src: src
                            .as_ptr()
                            .add(region.buffer_layout.offset as usize)
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(
                                region.texture_base.origin.x as usize
                                    + region.texture_base.origin.y as usize * dst_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        src_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize,
                        dst_pitch,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                    });
                }
            }
            TextureDimension::D3 => {
                let dst_pitch = dst.pixel_size * dst.size.width as usize;
                let dst_depth_pitch =
                    dst.pixel_size * dst.size.width as usize * dst.size.height as usize;
                for region in regions {
                    self.commands.push(Command::Copy3 {
                        src: src
                            .as_ptr()
                            .add(region.buffer_layout.offset as usize)
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .data
                            .as_ptr()
                            .add(
                                region.texture_base.origin.x as usize
                                    + region.texture_base.origin.y as usize * dst_pitch
                                    + region
                                        .texture_base
                                        .origin
                                        .z
                                        .max(region.texture_base.array_layer)
                                        as usize
                                        * dst_depth_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        src_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize,
                        dst_pitch,
                        src_depth_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize
                            * region.buffer_layout.rows_per_image.unwrap() as usize,
                        dst_depth_pitch,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                        depth: region.size.depth as usize,
                    });
                }
            }
        }
    }

    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &<Api as wgpu_hal::Api>::Texture,
        _src_usage: wgpu_hal::TextureUses,
        dst: &<Api as wgpu_hal::Api>::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = wgpu_hal::BufferTextureCopy>,
    {
        match src.dimension {
            TextureDimension::D1 => {
                for region in regions {
                    self.commands.push(Command::Copy {
                        src: core::slice::from_raw_parts_mut(
                            src.data
                                .as_ptr()
                                .add(region.texture_base.origin.x as usize * src.pixel_size)
                                .cast_mut()
                                .cast(),
                            region.size.width as usize * src.pixel_size,
                        ),
                        dst: dst
                            .as_ptr()
                            .add(region.buffer_layout.offset as usize)
                            .cast_mut()
                            .cast(),
                    });
                }
            }
            TextureDimension::D2 => {
                let src_pitch = src.pixel_size * src.size.width as usize;
                for region in regions {
                    self.commands.push(Command::Copy2 {
                        src: src
                            .data
                            .as_ptr()
                            .add(
                                region.texture_base.origin.x as usize
                                    + region.texture_base.origin.y as usize * src_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .as_ptr()
                            .add(region.buffer_layout.offset as usize)
                            .cast_mut()
                            .cast(),
                        src_pitch,
                        dst_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                    });
                }
            }
            TextureDimension::D3 => {
                let src_pitch = src.pixel_size * src.size.width as usize;
                let src_depth_pitch =
                    src.pixel_size * src.size.width as usize * src.size.height as usize;
                for region in regions {
                    self.commands.push(Command::Copy3 {
                        src: src
                            .data
                            .as_ptr()
                            .add(
                                region.texture_base.origin.x as usize
                                    + region.texture_base.origin.y as usize * src_pitch
                                    + region
                                        .texture_base
                                        .origin
                                        .z
                                        .max(region.texture_base.array_layer)
                                        as usize
                                        * src_depth_pitch,
                            )
                            .cast_mut()
                            .cast(),
                        dst: dst
                            .as_ptr()
                            .add(region.buffer_layout.offset as usize)
                            .cast_mut()
                            .cast(),
                        src_pitch,
                        dst_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize,
                        src_depth_pitch,
                        dst_depth_pitch: region.buffer_layout.bytes_per_row.unwrap() as usize
                            * region.buffer_layout.rows_per_image.unwrap() as usize,
                        rows: region.size.height as usize,
                        columns: region.size.width as usize,
                        depth: region.size.depth as usize,
                    });
                }
            }
        }
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

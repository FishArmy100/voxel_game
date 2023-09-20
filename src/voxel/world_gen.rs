use std::{borrow::Cow, sync::Arc};
use cgmath::Zero;
use wgpu::{PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry};
use wgpu::util::DeviceExt;
use crate::math::Vec3;
use crate::gpu::{GPUVec3, ShaderInfo};
use crate::utils::Array3D;
use crate::gpu::GBuffer;

use super::Voxel;

pub struct VoxelGenerator
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    chunk_size: Vec3<u32>,
    staging_buffer: GBuffer<i32>,
    storage_buffer: GBuffer<i32>,
    chunk_size_buffer: GBuffer<GPUVec3<u32>>,
    chunk_pos_buffer: GBuffer<GPUVec3<i32>>,

    bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

impl VoxelGenerator
{
    pub fn new(chunk_size: Vec3<u32>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, shader_info: ShaderInfo) -> Self 
    {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_info.source)),
        });

        let length = (chunk_size.x * chunk_size.y * chunk_size.z) as u64;

        let staging_buffer = GBuffer::<i32>::new_empty(length, wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, &device, Some("Staging buffer"));

        let storage_buffer_usage = wgpu::BufferUsages::STORAGE
                                 | wgpu::BufferUsages::COPY_DST
                                 | wgpu::BufferUsages::COPY_SRC;

        let storage_buffer = GBuffer::<i32>::new_empty(length, storage_buffer_usage, &device, Some("Storage buffer"));

        let uniform_usage = wgpu::BufferUsages::UNIFORM 
                          | wgpu::BufferUsages::COPY_DST
                          | wgpu::BufferUsages::COPY_SRC;

        let chunk_size_buffer = GBuffer::new(&[chunk_size.into()], uniform_usage, &device, Some("Chunk size buffer"));

        let chunk_pos_buffer = GBuffer::<GPUVec3<i32>>::new_empty(1, uniform_usage, &device, Some("Chunk position buffer"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry 
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None
                },
                BindGroupLayoutEntry 
                {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None
                },
                BindGroupLayoutEntry 
                {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None
                }]
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: shader_info.entry_point,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry 
                {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: chunk_size_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: chunk_pos_buffer.as_entire_binding()
                }],
        });

        Self 
        { 
            device, 
            queue, 
            chunk_size, 
            staging_buffer, 
            storage_buffer, 
            chunk_pos_buffer,
            chunk_size_buffer,
            bind_group, 
            compute_pipeline, 
        }
    }

    pub fn run(&mut self, chunk_pos: Vec3<i32>) -> Array3D<i32>
    {
        pollster::block_on(self.run_async(chunk_pos))
    }

    pub async fn run_async(&mut self, chunk_pos: Vec3<i32>) -> Array3D<i32>
    {
        self.chunk_pos_buffer.enqueue_set(&[chunk_pos.into()], &self.queue);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.insert_debug_marker("compute random numbers");
            compute_pass.dispatch_workgroups(self.chunk_size.x, self.chunk_size.y, self.chunk_size.z); // Number of cells to run, the (x,y,z) size of item being processed
        }

        self.storage_buffer.copy(&mut self.staging_buffer, &mut encoder);

        self.queue.submit(Some(encoder.finish()));
        
        
        let result = self.staging_buffer.read(&self.device);
        Array3D::from_vec(self.chunk_size.x as usize, self.chunk_size.y as usize, self.chunk_size.z as usize, result)
    }
}
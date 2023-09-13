use std::{borrow::Cow, sync::Arc};
use cgmath::Zero;
use wgpu::{PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry};
use wgpu::util::DeviceExt;
use crate::math::Vec3;
use crate::gpu::GPUVec3;
use crate::utils::Array3D;

use super::Voxel;

pub struct VoxelGenerator
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    chunk_size: Vec3<u32>,
    staging_buffer: wgpu::Buffer,
    storage_buffer: wgpu::Buffer,
    chunk_size_buffer: wgpu::Buffer,
    chunk_pos_buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

impl VoxelGenerator
{
    pub fn new(chunk_size: Vec3<u32>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self 
    {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/test_compute.wgsl"))),
        });

        let size = (std::mem::size_of::<u32>() * (chunk_size.x * chunk_size.y * chunk_size.z) as usize) as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0 as u8; size as usize]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let gpu_vec_size: GPUVec3<_> = chunk_size.into();
        let chunk_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[gpu_vec_size]),
            usage: wgpu::BufferUsages::UNIFORM 
                 | wgpu::BufferUsages::COPY_DST
                 | wgpu::BufferUsages::COPY_SRC
        });

        let chunk_pos_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[GPUVec3::<i32>::new(0, 0, 0)]),
            usage: wgpu::BufferUsages::UNIFORM 
                 | wgpu::BufferUsages::COPY_DST
                 | wgpu::BufferUsages::COPY_SRC
        });

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
            entry_point: "main",
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

    pub fn run(&self) -> Array3D<u32>
    {
        pollster::block_on(self.run_async())
    }

    pub async fn run_async(&self) -> Array3D<u32>
    {
        let voxel_count = self.chunk_size.x * self.chunk_size.y * self.chunk_size.z;
        let size = (std::mem::size_of::<u32>() * voxel_count as usize) as wgpu::BufferAddress;

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
        
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, size);

        self.queue.submit(Some(encoder.finish()));
        
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        let result: Vec<u32> = if let Some(Ok(())) = receiver.receive().await {
            let data = buffer_slice.get_mapped_range();
            let result = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            self.staging_buffer.unmap();

            result
        } else {
            panic!("failed to run compute on gpu!")
        };

        Array3D::from_vec(self.chunk_size.x as usize, self.chunk_size.y as usize, self.chunk_size.z as usize, result)
    }
}
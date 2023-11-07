use std::sync::Arc;
use wgpu::PipelineLayoutDescriptor;
use crate::math::Vec3;
use crate::gpu_utils::GPUVec3;
use crate::gpu_utils::bind_group::{MappedBuffer, Storage, Uniform, BindGroup, Entry};
use crate::utils::Array3D;

pub struct VoxelGenerator
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    chunk_size: Vec3<u32>,
    staging_buffer: MappedBuffer<i32>,
    storage_buffer: Storage<i32>,
    chunk_size_uniform: Uniform<GPUVec3<u32>>,
    chunk_position_uniform: Uniform<GPUVec3<i32>>,

    bind_group: BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

impl VoxelGenerator
{
    pub fn new(chunk_size: Vec3<u32>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self 
    {
        let cs_module = device.create_shader_module(wgpu::include_wgsl!("../shaders/test_compute.wgsl"));

        let length = (chunk_size.x * chunk_size.y * chunk_size.z) as u64;

        let staging_buffer = MappedBuffer::<i32>::with_capacity(length, wgpu::ShaderStages::COMPUTE, &device);
        let storage_buffer = Storage::<i32>::with_capacity(length, wgpu::ShaderStages::COMPUTE, &device);
        let chunk_size_uniform = Uniform::new(GPUVec3::from(chunk_size), wgpu::ShaderStages::COMPUTE, &device);
        let chunk_position_uniform = Uniform::<GPUVec3<i32>>::new_empty(wgpu::ShaderStages::COMPUTE, &device);

        let entries: &[&dyn Entry] = &[
            &storage_buffer, 
            &chunk_size_uniform, 
            &chunk_position_uniform
        ];

        let bind_group = BindGroup::new(entries, &device);

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group.layout()],
            push_constant_ranges: &[]
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

        Self 
        { 
            device, 
            queue, 
            chunk_size, 
            staging_buffer, 
            storage_buffer, 
            chunk_position_uniform,
            chunk_size_uniform,
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
        self.chunk_position_uniform.enqueue_write(chunk_pos.into(), &self.queue);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group.bind_group(), &[]);
            compute_pass.insert_debug_marker("compute random numbers");
            compute_pass.dispatch_workgroups(self.chunk_size.x, self.chunk_size.y, self.chunk_size.z); // Number of cells to run, the (x,y,z) size of item being processed
        }

        self.storage_buffer.copy_to_mapped(&mut self.staging_buffer, &mut encoder);

        self.queue.submit(Some(encoder.finish()));
        
        let result = self.staging_buffer.read(&self.device);
        Array3D::from_vec(self.chunk_size.x as usize, self.chunk_size.y as usize, self.chunk_size.z as usize, result)
    }
}
use std::sync::Arc;

use glam::{IVec3, UVec3, IVec4};
use vox_core::terrain::TerrainArgs;
use wgpu::include_spirv;

use crate::{prelude::Array3D, gpu_utils::{BindGroup, Uniform, Storage, MappedBuffer}, utils::{Wrappable, Wrapper}};

unsafe impl Wrappable for TerrainArgs {}

pub struct TerrainGenerator
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    pipeline: wgpu::ComputePipeline, 
    bind_group: BindGroup,
    args_buffer: Uniform<Wrapper<TerrainArgs>>,
    terrain_args: TerrainArgs,
    position_buffer: Uniform<IVec4>,

    staging_buffer: MappedBuffer<u32>,
    storage_buffer: Storage<u32>
}

impl TerrainGenerator
{
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, args: TerrainArgs) -> Self 
    { 
        let args_buffer = Uniform::new(Wrapper(args), wgpu::ShaderStages::COMPUTE, &device);
        
        let data = vec![0 as u32; (args.chunk_size.pow(3)) as usize];
        let staging_buffer = MappedBuffer::new(&data, wgpu::ShaderStages::COMPUTE, &device);
        let storage_buffer = Storage::new(&data, wgpu::ShaderStages::COMPUTE, &device);

        let position_buffer = Uniform::new(IVec4::ZERO, wgpu::ShaderStages::COMPUTE, &device);

        let bind_group = BindGroup::new(&[&storage_buffer, &args_buffer, &position_buffer], &device);
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group.layout()],
            push_constant_ranges: &[]
        });

        let shader = &device.create_shader_module(include_spirv!(env!("terrain_gen.spv")));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Terrain Generator Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: shader,
            entry_point: "cs_main",
        });

        Self 
        {
            device,
            queue,
            pipeline,
            bind_group,
            terrain_args: args,
            args_buffer,
            position_buffer,
            staging_buffer,
            storage_buffer
        }
    }

    pub fn generate(&mut self, chunk_position: IVec3) -> Array3D<u32>
    {
        let chunk_size = self.terrain_args.chunk_size;
        
        self.position_buffer.enqueue_write(chunk_position.extend(0), &self.queue);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group.bind_group(), &[]);
            compute_pass.dispatch_workgroups(chunk_size, chunk_size, chunk_size); // Number of cells to run, the (x,y,z) size of item being processed
        }

        self.storage_buffer.copy_to_mapped(&mut self.staging_buffer, &mut encoder);

        self.queue.submit(Some(encoder.finish()));
        
        let result = self.staging_buffer.read(&self.device);
        Array3D::from_vec(chunk_size as usize, chunk_size as usize, chunk_size as usize, result)
    }
}
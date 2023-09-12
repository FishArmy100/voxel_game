use std::cell::Cell;
use std::{sync::Arc, borrow::Cow};

use crate::math::Vec3;

use crate::gpu::bind_group::IBindGroup;

use super::GPUBuffer;
use super::bind_group::StorageBindGroup;

pub trait ComputeStage
{
    type Result;
    type Args;
    fn on_begin_execute<'pass, 's: 'pass>(&'s mut self, args: Self::Args, queue: &wgpu::Queue) -> Vec3<u32>;
    fn on_finish_execute(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder);
    fn bind_groups(&self) -> Box<[&dyn IBindGroup]>;
    fn on_collect_result(&self) -> Self::Result;
}

pub struct ComputeShaderInfo<'a>
{
    pub source: &'a str,
    pub main: &'a str
}

pub struct ComputeExecutor<T> where T : ComputeStage
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    stage: T,

    pipeline: wgpu::ComputePipeline
}

impl<T> ComputeExecutor<T> where T : ComputeStage
{
    pub fn new(stage: T, shader_info: ComputeShaderInfo, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self
    {
        let pipeline = Self::get_pipeline(&stage, shader_info, &device);

        Self 
        { 
            device, 
            queue, 
            stage, 
            pipeline 
        }
    }

    pub fn execute(&mut self, args: T::Args) -> T::Result
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: None});
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

            cpass.set_pipeline(&self.pipeline);

            let workgroup_count = self.stage.on_begin_execute(args, &self.queue);

            let bind_groups = &self.stage.bind_groups();
            {
                for index in 0..bind_groups.len()
                {
                    cpass.set_bind_group(index as u32, bind_groups[index].bind_group_handle(), &[]);
                }
            }

            cpass.dispatch_workgroups(workgroup_count.x, workgroup_count.y, workgroup_count.z);
        }

        self.stage.on_finish_execute(&self.queue, &mut encoder);
        self.queue.submit(Some(encoder.finish()));

        self.stage.on_collect_result()
    }

    fn get_pipeline(stage: &T, shader_info: ComputeShaderInfo, device: &wgpu::Device) -> wgpu::ComputePipeline
    {
        let bind_group_layouts: Vec<_> = stage.bind_groups().iter()
            .map(|b| b.layout())
            .collect();
        
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: None, 
            bind_group_layouts: &bind_group_layouts, 
            push_constant_ranges: &[] 
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { 
            label: None, 
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_info.source)) 
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &shader,
            entry_point: shader_info.main
        });

        pipeline
    }
}

pub struct TestComputeStage
{
    staging_buffer: GPUBuffer<u32>,
    storage: StorageBindGroup<u32>
}

impl TestComputeStage
{
    pub fn new(device: Arc<wgpu::Device>) -> Self
    {
        let staging_buffer_usage = wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST;
        let staging_buffer = GPUBuffer::with_capacity(1, staging_buffer_usage, None, device.clone());

        let storage = StorageBindGroup::new(&[5], wgpu::ShaderStages::COMPUTE, "test storage".into(), device);

        Self 
        { 
            staging_buffer,
            storage
        }
    }
}

impl ComputeStage for TestComputeStage
{
    type Result = Vec<u32>;
    type Args = Vec<u32>;

    fn on_begin_execute<'pass, 's: 'pass>(&'s mut self, args: Self::Args, queue: &wgpu::Queue) -> Vec3<u32> 
    {
        self.storage.enqueue_set_data(&args, queue);
        self.staging_buffer.enqueue_set_data(&vec![0; args.len()], queue);
        Vec3::new(args.len() as u32, 0, 0)
    }

    fn on_finish_execute(&self, _queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) 
    {
        encoder.copy_buffer_to_buffer(&self.storage.buffer().buffer(), 0, self.staging_buffer.buffer(), 0, self.storage.buffer().len() * std::mem::size_of::<u32>() as u64);
    }

    fn bind_groups(&self) -> Box<[&dyn IBindGroup]> 
    {
        Box::new([&self.storage])
    }

    fn on_collect_result(&self) -> Self::Result 
    {
        self.staging_buffer.read()
    }
}


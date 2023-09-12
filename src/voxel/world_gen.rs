use std::{borrow::Cow, sync::Arc};
use cgmath::Zero;
use wgpu::util::DeviceExt;
use crate::math::Vec3;

pub struct ChunkGenerator
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    chunk_size: Vec3<usize>,
    staging_buffer: wgpu::Buffer,
    storage_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,

    gen_count: u32 // TEMP
}

impl ChunkGenerator
{
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, gen_count: u32) -> Self 
    {
        // Loads the shader from WGSL
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/test_compute.wgsl"))),
        });

        let size = (std::mem::size_of::<f32>() * gen_count as usize) as wgpu::BufferAddress;

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

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            entry_point: "main",
        });

        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

        Self 
        { 
            device, 
            queue, 
            chunk_size: Vec3::zero(), 
            staging_buffer, 
            storage_buffer, 
            bind_group, 
            compute_pipeline, 
            gen_count
        }
    }

    pub fn run(&self)
    {
        pollster::block_on(self.run_async())
    }

    pub async fn run_async(&self)
    {
        let size = (std::mem::size_of::<f32>() * self.gen_count as usize) as wgpu::BufferAddress;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });

            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.insert_debug_marker("compute random numbers");
            cpass.dispatch_workgroups(self.gen_count, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }
        
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, size);

        self.queue.submit(Some(encoder.finish()));
        
        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        let result = if let Some(Ok(())) = receiver.receive().await {
            let data = buffer_slice.get_mapped_range();
            let result = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            self.staging_buffer.unmap();

            result
        } else {
            panic!("failed to run compute on gpu!")
        };

        print_result(&result);
    }
}

fn print_result(result: &Vec<f32>)
{
    let disp_steps: Vec<String> = result
        .iter()
        .map(|n| n.to_string())
        .collect();

    println!("Steps: [{}]", disp_steps.join(", "));
}
use crate::{math::{Color, Vec2}, rendering::{RenderStage, get_command_encoder}, gpu_utils::{Uniform, BindGroup, Entry}};

pub enum Visibility { Opaque, Empty }

pub struct VoxelIndex(u16);

pub struct Voxel 
{
    pub color: Color,
    pub name: &'static str,
    pub visibility: Visibility
}

pub struct VoxelRenderer
{
    pipeline: wgpu::ComputePipeline,
    screen_size: Vec2<u32>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl VoxelRenderer
{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let cs_module = device.create_shader_module(wgpu::include_spirv!(env!("voxel_shader.spv")));
        let screen_size = Vec2::new(config.width, config.height);

        let texture_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture { 
                access: wgpu::StorageTextureAccess::ReadWrite, 
                format: wgpu::TextureFormat::Rgba32Float, 
                view_dimension: wgpu::TextureViewDimension::D2 
            },
            count: None
        };

        let bind_group_layout = BindGroup::construct_layout_from_entries(&[texture_entry], device);

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
            label: Some("Voxel Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main"
        });

        Self 
        {
            pipeline,
            screen_size,
            bind_group_layout,
        }
    }
}

impl RenderStage for VoxelRenderer
{
    fn on_draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, _depth_texture: &crate::gpu_utils::Texture) 
    {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Voxel Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view)
                }
            ]
        });

        let mut encoder = get_command_encoder(device);
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Voxel Compute Pass"),
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.insert_debug_marker("trying to raytrace voxels");
            compute_pass.dispatch_workgroups(self.screen_size.x, self.screen_size.y, 1); 
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
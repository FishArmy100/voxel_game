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

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,

    bind_group: wgpu::BindGroup,
}

impl VoxelRenderer
{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let cs_module = device.create_shader_module(wgpu::include_spirv!(env!("voxel_shader.spv")));

        let (texture, texture_view) = Self::make_texture(device, config);

        let texture_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture { 
                sample_type: wgpu::TextureSampleType::Float { filterable: true /* ??? */ }, 
                view_dimension: wgpu::TextureViewDimension::D2, 
                multisampled: false
            },
            count: None
        };

        let bind_group_layout = BindGroup::construct_layout_from_entries(&[texture_size_x_uniform.get_layout(0), texture_size_y_uniform.get_layout(1), texture_entry], device);

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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Voxel Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                }
            ]
        });

        Self 
        {
            pipeline,
            screen_size: Vec2::new(config.width, config.height),
            texture,
            texture_view,
            bind_group
        }
    }

    fn make_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView)
    {
        let texture_size = wgpu::Extent3d { 
            width: config.width, 
            height: config.height, 
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Voxel Indirection Buffer"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}

impl RenderStage for VoxelRenderer
{
    fn on_draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, depth_texture: &crate::gpu_utils::Texture) 
    {
        let mut encoder = get_command_encoder(device);
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Voxel Compute Pass"),
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.insert_debug_marker("trying to raytrace voxels");
            compute_pass.dispatch_workgroups(self.screen_size.x, self.screen_size.y, 1); 
        }

        encoder.copy_texture_to_texture(self.texture.as_image_copy(), vi, copy_size)
    }
}
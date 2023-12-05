use glam::UVec2;
use wgpu::*;

use crate::{math::{Color}, rendering::{RenderStage, get_command_encoder, construct_render_pipeline, RenderPipelineInfo, get_render_pass, camera::Camera}, gpu_utils::{Uniform, Entry}};
use glam::Vec4;

pub enum Visibility { Opaque, Empty }

pub struct VoxelIndex(u16);

pub struct Voxel 
{
    pub color: Color,
    pub name: &'static str,
    pub visibility: Visibility
}

struct VoxelRendererData
{
    width_uniform: Uniform<u32>,
    height_uniform: Uniform<u32>,

    camera_eye: Uniform<Vec4>,
    camera_target: Uniform<Vec4>,
    camera_fov: Uniform<f32>
}

pub struct VoxelRenderer
{
    compute_pipeline: wgpu::ComputePipeline,
    indirect_texture: wgpu::Texture,
    compute_bind_group: wgpu::BindGroup,
    compute_bind_group_layout: wgpu::BindGroupLayout,

    render_pipeline: wgpu::RenderPipeline,
    indirect_texture_view: wgpu::TextureView,
    indirect_texture_sampler: wgpu::Sampler,
    render_bind_group: wgpu::BindGroup,
    render_bind_group_layout: wgpu::BindGroupLayout,

    data: VoxelRendererData,

    screen_size: UVec2
}

impl VoxelRenderer
{
    pub fn new(device: &wgpu::Device, camera: &Camera, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let texture = get_texture(device, config);
        let view = get_texture_view(&texture);
        let sampler = get_sampler(device);

        let render_bind_group_layout = create_render_bind_group_layout(device);
        let render_bind_group = create_render_bind_group(device, &render_bind_group_layout, &view, &sampler);
        let render_shader = &device.create_shader_module(include_spirv!(env!("screen_shader.spv")));

        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo { 
            shader: render_shader, 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[],
            bind_groups: &[&render_bind_group_layout],
            label: None
        });

        let width_uniform = Uniform::new(config.width, ShaderStages::COMPUTE, device);
        let height_uniform = Uniform::new(config.height, ShaderStages::COMPUTE, device);

        let camera_eye = Uniform::new(camera.eye.extend(0.0), ShaderStages::COMPUTE, device);
        let camera_target = Uniform::new(camera.target.extend(0.0), ShaderStages::COMPUTE, device);
        let camera_fov = Uniform::new(camera.fov, ShaderStages::COMPUTE, device);
    

        let data = VoxelRendererData { 
            width_uniform,
            height_uniform,
            camera_eye,
            camera_target,
            camera_fov
        };
        
        let compute_bind_group_layout = create_compute_bind_group_layout(device, &data);
        let compute_bind_group = create_compute_bind_group(device, &compute_bind_group_layout, &view, &data);

        // let compute_shader = &device.create_shader_module(include_spirv!(env!("raytracing_shader.spv")));
        let compute_shader = &device.create_shader_module(include_wgsl!("raytracing_shader.wgsl"));

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { 
            label: None,
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[]
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor { 
            label: None, 
            layout: Some(&compute_pipeline_layout), 
            module: &compute_shader, 
            entry_point: "cs_main"
        });

        Self 
        { 
            compute_pipeline, 
            indirect_texture: texture, 
            compute_bind_group, 
            compute_bind_group_layout, 
            render_pipeline, 
            indirect_texture_view: view, 
            indirect_texture_sampler: sampler, 
            render_bind_group, 
            render_bind_group_layout,
            data,
            screen_size: UVec2::new(config.width, config.height)
        }
    }

    pub fn resize(&mut self, queue: &Queue, device: &Device, config: &SurfaceConfiguration)
    {
        self.data.width_uniform.enqueue_write(config.width, queue);
        self.data.height_uniform.enqueue_write(config.height, queue);

        self.indirect_texture = get_texture(device, config);
        self.indirect_texture_view = get_texture_view(&self.indirect_texture);
        self.render_bind_group = create_render_bind_group(device, &self.render_bind_group_layout, &self.indirect_texture_view, &self.indirect_texture_sampler);
        self.compute_bind_group = create_compute_bind_group(device, &self.compute_bind_group_layout, &self.indirect_texture_view, &self.data);
        self.screen_size = UVec2::new(config.width, config.height);
    }

    pub fn update(&mut self, camera: &Camera, queue: &Queue)
    {
        self.data.camera_eye.enqueue_write(camera.eye.extend(0.0), queue);
        self.data.camera_target.enqueue_write(camera.target.extend(0.0), queue);
        self.data.camera_fov.enqueue_write(camera.fov, queue);
    }
}

impl RenderStage for VoxelRenderer
{
    fn on_draw(&mut self, device: &Device, queue: &Queue, view: &TextureView, depth_texture: &crate::gpu_utils::Texture) 
    {
        let mut encoder = get_command_encoder(device);
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor { 
                label: None 
            });
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.dispatch_workgroups(self.screen_size.x, self.screen_size.y, 1);
        }
        {
            let mut render_pass = get_render_pass(&mut encoder, view, Some(depth_texture));
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

fn create_compute_bind_group(device: &Device, layout: &BindGroupLayout, view: &TextureView, data: &VoxelRendererData) -> BindGroup
{
    let bind_group = device.create_bind_group(&BindGroupDescriptor { 
        label: None, 
        layout, 
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(view)
            },
            BindGroupEntry {
                binding: 1,
                resource: data.width_uniform.get_resource()
            },
            BindGroupEntry {
                binding: 2,
                resource: data.height_uniform.get_resource()
            },
            BindGroupEntry {
                binding: 3,
                resource: data.camera_eye.get_resource()
            },
            BindGroupEntry {
                binding: 4,
                resource: data.camera_target.get_resource()
            },
            BindGroupEntry {
                binding: 5,
                resource: data.camera_fov.get_resource()
            },
        ] 
    });

    bind_group
}

fn create_compute_bind_group_layout(device: &Device, data: &VoxelRendererData) -> BindGroupLayout
{
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { 
        label: None, 
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture { 
                    access: StorageTextureAccess::ReadWrite, 
                    format: TextureFormat::Rgba8Unorm,
                    view_dimension: TextureViewDimension::D2
                },
                count: None
            },
            data.width_uniform  .get_layout(1),
            data.height_uniform .get_layout(2),
            data.camera_eye     .get_layout(3),
            data.camera_target  .get_layout(4),
            data.camera_fov     .get_layout(5),
        ] 
    });

    layout
}


fn create_render_bind_group(device: &Device, layout: &BindGroupLayout, view: &TextureView, sampler: &Sampler) -> BindGroup
{
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(view)
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler)
            }
        ]
    });

    bind_group
}

fn create_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture { 
                    sample_type: TextureSampleType::Float { filterable: true }, 
                    view_dimension: TextureViewDimension::D2, 
                    multisampled: false
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None
            }
        ]
    });

    layout
}

fn get_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture
{
    let size = Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[]
    });

    texture
}

fn get_texture_view(texture: &wgpu::Texture) -> wgpu::TextureView
{
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn get_sampler(device: &wgpu::Device) -> wgpu::Sampler
{
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    sampler
}
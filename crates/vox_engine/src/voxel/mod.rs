use glam::UVec2;
use vox_core::RTCameraInfo;
use wgpu::*;
use wgpu_profiler::{wgpu_profiler, GpuProfiler};

use crate::{math::{Color}, rendering::{RenderStage, get_command_encoder, construct_render_pipeline, RenderPipelineInfo, get_render_pass, camera::Camera}, gpu_utils::{Uniform, Entry, WgpuState}, prelude::FrameState};
use glam::Vec4;

pub enum Visibility { Opaque, Empty }

#[derive(Clone, Copy)]
struct RTWrapper(RTCameraInfo);

unsafe impl bytemuck::Pod for RTWrapper {}
unsafe impl bytemuck::Zeroable for RTWrapper {}

pub struct VoxelIndex(u16);

pub struct Voxel 
{
    pub color: Color,
    pub name: &'static str,
    pub visibility: Visibility
}

pub struct VoxelRenderer
{
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    render_bind_group_layout: wgpu::BindGroupLayout,

    rt_camera_uniform: Uniform<RTWrapper>,

    current_camera: RTCameraInfo,

    profiler: GpuProfiler,
}

impl VoxelRenderer
{
    pub fn new(gpu_state: &WgpuState, camera: &Camera) -> Self 
    {
        let device = &gpu_state.device();
        let config = gpu_state.surface_config();

        let rt_info = camera.get_rt_info(config.width, config.height);
        let rt_camera_uniform = Uniform::new(RTWrapper(rt_info), ShaderStages::COMPUTE, device);

        let render_bind_group_layout = create_render_bind_group_layout(device);
        let render_bind_group = create_render_bind_group(device, &render_bind_group_layout, &rt_camera_uniform);
        let render_shader = &device.create_shader_module(include_spirv!(env!("screen_shader.spv")));

        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo { 
            shader: render_shader, 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[],
            bind_groups: &[&render_bind_group_layout],
            label: None
        });

        let profiler = GpuProfiler::new(gpu_state.adapter(), device, &gpu_state.queue(), 4);

        Self 
        {
            render_pipeline,
            render_bind_group, 
            render_bind_group_layout,
            rt_camera_uniform,
            current_camera: rt_info,
            profiler
        }
    }

    pub fn resize(&mut self, queue: &Queue, device: &Device, config: &SurfaceConfiguration)
    {
        self.current_camera.width = config.width;
        self.current_camera.height = config.height;
        self.rt_camera_uniform.enqueue_write(RTWrapper(self.current_camera), queue);
        self.render_bind_group = create_render_bind_group(device, &self.render_bind_group_layout, &self.rt_camera_uniform);
    }

    pub fn update(&mut self, camera: &Camera, queue: &Queue)
    {
        let rt_info = camera.get_rt_info(self.current_camera.width, self.current_camera.height);
        self.rt_camera_uniform.enqueue_write(RTWrapper(rt_info), queue);
        self.current_camera = rt_info;
    }

    pub fn get_profiling_info(&mut self) -> Option<f32>
    {
        if let Some(profiling_data) = self.profiler.process_finished_frame() 
        {
            let range = profiling_data.first().unwrap().time.clone();
            let time = (range.end - range.start) as f32;
            println!("gpu time for {}: {}ms", profiling_data.first().unwrap().label, time * 1000.0);
            Some(time)
        }
        else 
        {
            None
        }
    }
}

impl RenderStage for VoxelRenderer
{
    fn on_draw(&mut self, device: &Device, queue: &Queue, view: &TextureView, depth_texture: &crate::gpu_utils::Texture) 
    {
        let mut encoder = get_command_encoder(device);
        wgpu_profiler!("voxel renderer scope", &mut self.profiler, &mut encoder, &device, {
            let mut render_pass = get_render_pass(&mut encoder, view, Some(depth_texture));
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        });

        self.profiler.resolve_queries(&mut encoder);

        queue.submit(Some(encoder.finish()));
        self.profiler.end_frame().unwrap();
    }
}


fn create_render_bind_group(device: &Device, layout: &BindGroupLayout, camera_uniform: &Uniform<RTWrapper>) -> BindGroup
{
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: camera_uniform.get_resource()
            },
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
                ty: BindingType::Buffer { 
                    ty: BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None,
            },
        ]
    });

    layout
}
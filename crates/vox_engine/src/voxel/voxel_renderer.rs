use glam::vec3;
use vox_core::{camera::RTCameraInfo, VoxelModelInstance};
use wgpu::*;
use wgpu_profiler::{wgpu_profiler, GpuProfiler};
use super::{*, prefab::VoxelPrefab};

use crate::{
    rendering::{
        RenderStage, 
        get_command_encoder, 
        construct_render_pipeline, 
        RenderPipelineInfo, 
        get_render_pass, 
        camera::Camera
    }, 
    gpu_utils::{
        Uniform, 
        WgpuState, 
        Storage, 
        BindGroup
    }, utils::Wrapper,
};

pub struct VoxelRenderer
{
    render_pipeline: wgpu::RenderPipeline,
    bind_group: BindGroup,

    rt_camera_uniform: Uniform<Wrapper<RTCameraInfo>>,
    current_camera: RTCameraInfo,

    profiler: GpuProfiler,
}

impl VoxelRenderer
{
    pub fn new(gpu_state: &WgpuState, camera: &Camera, voxels: &[u32], instances: &[Wrapper<VoxelModelInstance>]) -> Self 
    {
        let device = gpu_state.device();
        let config = gpu_state.surface_config();

        let rt_info = camera.get_rt_info(config.width, config.height);
        let rt_camera_uniform = Uniform::new(Wrapper(rt_info), ShaderStages::FRAGMENT, device);

        let instance_storage = Storage::new(instances, wgpu::ShaderStages::FRAGMENT, &device);
        let voxel_storage = Storage::new(voxels, wgpu::ShaderStages::FRAGMENT, &device);
        let voxel_color_storage = Storage::new(&voxel_colors(), wgpu::ShaderStages::FRAGMENT, &device);

        let bind_group = BindGroup::new(&[&rt_camera_uniform, &instance_storage, &voxel_storage, &voxel_color_storage], device);
       
        let render_shader = &device.create_shader_module(include_spirv!(env!("voxel_raytracer.spv")));

        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo { 
            shader: render_shader, 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[],
            bind_groups: &[&bind_group.layout()],
            label: None
        });

        let profiler = GpuProfiler::new(gpu_state.adapter(), device, &gpu_state.queue(), 4);

        Self 
        {
            render_pipeline,
            bind_group,
            rt_camera_uniform,
            current_camera: rt_info,
            profiler
        }
    }

    pub fn register_prefabs(&mut self, prefabs: &[VoxelPrefab])
    {
        todo!()
    }

    /// Panics if the name of the prefab isn't loaded in
    pub fn spawn_prefab<'a, T>(&mut self, name: T) 
        where T : Into<&'a str>
    {
        todo!()
    }

    pub fn resize(&mut self, queue: &Queue, device: &Device, config: &SurfaceConfiguration)
    {
        self.current_camera.width = config.width;
        self.current_camera.height = config.height;
        self.rt_camera_uniform.enqueue_write(Wrapper(self.current_camera), queue);
    }

    pub fn update(&mut self, camera: &Camera, queue: &Queue)
    {
        let rt_info = camera.get_rt_info(self.current_camera.width, self.current_camera.height);
        self.rt_camera_uniform.enqueue_write(Wrapper(rt_info), queue);
        self.current_camera = rt_info;
    }

    pub fn get_profiling_info(&mut self) -> Option<f32>
    {
        if let Some(profiling_data) = self.profiler.process_finished_frame() 
        {
            let range = profiling_data.first().unwrap().time.clone();
            let time = (range.end - range.start) as f32;
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
            render_pass.set_bind_group(0, &self.bind_group.bind_group(), &[]);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        });

        self.profiler.resolve_queries(&mut encoder);

        queue.submit(Some(encoder.finish()));
        self.profiler.end_frame().unwrap();
    }
}
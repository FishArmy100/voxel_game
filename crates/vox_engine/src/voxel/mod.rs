use glam::{UVec2, Vec3, uvec3};
use vox_core::{camera::RTCameraInfo, VoxelVolume, utils::flatten_index};
use wgpu::*;
use wgpu_profiler::{wgpu_profiler, GpuProfiler};

use crate::{
    math::Color, 
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
        Entry, 
        WgpuState, GBuffer, Storage, BindGroup
    }, prelude::Array3D, utils::{Wrapper, Wrappable},
};

unsafe impl Wrappable for RTCameraInfo {}
unsafe impl Wrappable for VoxelVolume {}

pub struct VoxelRenderer
{
    render_pipeline: wgpu::RenderPipeline,
    bind_group: BindGroup,

    rt_camera_uniform: Uniform<Wrapper<RTCameraInfo>>,
    current_camera: RTCameraInfo,

    // Temp
    voxel_storage: Storage<u32>,
    volume_uniform: Uniform<Wrapper<VoxelVolume>>,

    profiler: GpuProfiler,
}

impl VoxelRenderer
{
    pub fn new(gpu_state: &WgpuState, camera: &Camera) -> Self 
    {
        let device = gpu_state.device();
        let config = gpu_state.surface_config();

        let rt_info = camera.get_rt_info(config.width, config.height);
        let rt_camera_uniform = Uniform::new(Wrapper(rt_info), ShaderStages::FRAGMENT, device);

        let (volume, voxels) = build_vox_model(include_bytes!("../../resources/teapot.vox"), Vec3::ZERO, 1.0, |i| {
            if i == 84
            {
                2
            }
            else
            {
                1
            }
        }).unwrap();
        println!("Volume: \n\t- Size: [{}, {}, {}];", volume.dim_x(), volume.dim_y(), volume.dim_z());

        let volume_uniform = Uniform::new(Wrapper(volume), wgpu::ShaderStages::FRAGMENT, &device);
        let voxel_storage = Storage::new(voxels.as_slice(), wgpu::ShaderStages::FRAGMENT, &device);

        let bind_group = BindGroup::new(&[&rt_camera_uniform, &volume_uniform, &voxel_storage], device);
       
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
            volume_uniform,
            voxel_storage,
            profiler
        }
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

pub fn build_vox_model<F>(bytes: &[u8], origin: Vec3, voxel_size: f32, mut index_converter: F) -> Result<(VoxelVolume, Array3D<u32>), &'static str>
    where F : FnMut(u8) -> u32
{
    let data = match dot_vox::load_bytes(bytes)
    {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let model = match data.models.first()
    {
        Some(m) => m,
        None => return Err(".vox data does not have a model"),
    };

    let volume = VoxelVolume::new(origin, voxel_size, model.size.x, model.size.z, model.size.y);
    let mut voxel_array = Array3D::new_with_value(volume.dim_x() as usize, volume.dim_y() as usize, volume.dim_z() as usize, 0);
    for v in &model.voxels
    {
        let index = (v.x as usize, v.z as usize, v.y as usize);
        voxel_array[index] = index_converter(v.i);
    }

    Ok((volume, voxel_array))
}
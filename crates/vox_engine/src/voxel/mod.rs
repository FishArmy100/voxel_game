use glam::{UVec2, Vec3, uvec3, vec3, Vec4, vec4};
use vox_core::{camera::RTCameraInfo, VoxelModelInstance, utils::flatten_index, VoxelModel};
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
    }, prelude::Array3D, utils::{Wrapper, Wrappable },
};

unsafe impl Wrappable for RTCameraInfo {}
unsafe impl Wrappable for VoxelModelInstance {}

#[derive(Debug, Clone, Copy)]
pub struct Voxel 
{
    pub name: &'static str,
    pub id: u32,
    pub color: Vec4
}

pub const VOXELS: &[Voxel] = &[
    Voxel {
        name: "air",
        id: 0,
        color: vec4(1.0, 1.0, 1.0, 1.0)
    },
    Voxel {
        name: "dirt",
        id: 1,
        color: vec4(69.0 / 255.0, 45.0 / 255.0, 45.0 / 255.0, 1.0)
    },
    Voxel {
        name: "grass",
        id: 2,
        color: vec4(93.0 / 255.0, 146.0 / 255.0, 77.0 / 255.0, 1.0)
    },
    Voxel {
        name: "granite",
        id: 3,
        color: vec4(136.0 / 255.0, 140.0 / 255.0, 141.0 / 255.0, 1.0)
    },
    Voxel {
        name: "sandstone",
        id: 4,
        color: vec4(184.0 / 255.0, 176.0 / 255.0, 155.0 / 255.0, 1.0)
    },
    Voxel {
        name: "tree bark",
        id: 5,
        color: vec4(105.0 / 255.0, 75.0 / 255.0, 53.0 / 255.0, 1.0)
    },
    Voxel {
        name: "tree leaves",
        id: 6,
        color: vec4(95.0 / 255.0, 146.0 / 255.0, 106.0 / 255.0, 1.0)
    },
    Voxel {
        name: "water",
        id: 7,
        color: vec4(28.0 / 255.0, 163.0 / 255.0, 236.0 / 255.0, 1.0)
    },
    Voxel {
        name: "error",
        id: 8,
        color: vec4(1.0, 0.0, 1.0, 1.0)
    }
];

pub const AIR:          &Voxel = &VOXELS[0];
pub const DIRT:         &Voxel = &VOXELS[1];
pub const GRASS:        &Voxel = &VOXELS[2];
pub const GRANITE:      &Voxel = &VOXELS[3];
pub const SANDSTONE:    &Voxel = &VOXELS[4];
pub const TREE_BARK:    &Voxel = &VOXELS[5];
pub const TREE_LEAVES:  &Voxel = &VOXELS[6];
pub const WATER:        &Voxel = &VOXELS[7];
pub const ERROR:        &Voxel = &VOXELS[8];

pub fn voxel_colors() -> Vec<Vec4>
{
    VOXELS.iter().map(|v| v.color).collect()
}

pub struct VoxelRenderer
{
    render_pipeline: wgpu::RenderPipeline,
    bind_group: BindGroup,

    rt_camera_uniform: Uniform<Wrapper<RTCameraInfo>>,
    current_camera: RTCameraInfo,

    // Temp
    voxel_storage: Storage<u32>,
    instance_storage: Storage<Wrapper<VoxelModelInstance>>,

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

        let vox_files: [&[u8]; 3] = 
        [
            include_bytes!("../../resources/teapot.vox"),
            include_bytes!("../../resources/3x3x3.vox"),
            include_bytes!("../../resources/monu2.vox"),
        ];

        let (models, voxels) = load_voxel_models(&vox_files, |i| {
            let i = i + 1;
            match i
            {
                121 => SANDSTONE.id,
                122 => TREE_BARK.id,
                123 => GRANITE.id,
                81 => TREE_LEAVES.id,
                97 => WATER.id,
                _ => ERROR.id
            }
        }).unwrap();
        
        
        let teapot = models[0];
        let holy_cube = models[1];
        let monument = models[2];

        let teapot_instance = VoxelModelInstance::new(vec3(24.0, 24.0, 48.0), 1.0 / 16.0, teapot);
        let monument_instance = VoxelModelInstance::new(vec3(0.0, 0.0, 0.0), 1.0, monument);

        let instances = &[Wrapper(monument_instance)/*, Wrapper(teapot_instance) */];

        let instance_storage = Storage::new(instances, wgpu::ShaderStages::FRAGMENT, &device);
        let voxel_storage = Storage::new(voxels.as_slice(), wgpu::ShaderStages::FRAGMENT, &device);
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
            instance_storage,
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

pub fn build_vox_model<F>(bytes: &[u8], start_index: u32, mut index_converter: F) -> Result<(VoxelModel, Array3D<u32>), &'static str>
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

    let mut unique = vec![];
    
    let voxel_model = VoxelModel::new(model.size.x, model.size.z, model.size.y, start_index);
    let mut voxel_array = Array3D::new_with_value(voxel_model.dim_x() as usize, voxel_model.dim_y() as usize, voxel_model.dim_z() as usize, 0);
    for v in &model.voxels
    {
        if !unique.contains(&(v.i as u32))
        {
            unique.push(v.i as u32)
        }

        let index = (v.x as usize, v.z as usize, v.y as usize);
        voxel_array[index] = index_converter(v.i);
    }

    println!("unique voxel ids: {:?}", unique);

    Ok((voxel_model, voxel_array))
}

pub fn load_voxel_models<F>(vox_files: &[&[u8]], mut index_converter: F) -> Result<(Vec<VoxelModel>, Vec<u32>), &'static str>
    where F : FnMut(u8) -> u32
{
    let mut models = vec![];
    let mut voxels = vec![];

    for f in vox_files
    {
        let (model, vs) = build_vox_model(f, voxels.len() as u32, &mut index_converter)?;
        models.push(model);
        voxels.extend(vs.as_slice());
    }

    Ok((models, voxels))
}
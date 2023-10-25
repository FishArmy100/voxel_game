use std::{sync::Arc, cell::RefCell};

use std::sync::{Mutex, MutexGuard};

use crate::rendering::{get_command_encoder, RenderPassInfo, build_render_pass};
use crate::{math::{Vec3, Color}, rendering::{construct_render_pipeline, RenderPipelineInfo, RenderStage}, camera::{Camera, CameraUniform}};
use crate::gpu_utils::{BindGroup, Uniform, VertexBuffer, VertexData, GPUVec3, IndexBuffer, GPUVec4};
use crate::voxel::voxel_rendering::*;

use super::{terrain::VoxelTerrain, VoxelStorage, Voxel};

pub struct ChunkRenderData
{
    face_instance_buffer: VertexBuffer<VoxelFace>
}

impl ChunkRenderData
{
    pub fn face_instance_buffer(&self) -> &VertexBuffer<VoxelFace> { &self.face_instance_buffer }

    pub fn new(mesh: &VoxelMesh, device: &wgpu::Device) -> Self
    {
        Self 
        {
            face_instance_buffer: mesh.create_buffers(device)
        }
    }
}

pub struct TerrainRenderStage<TStorage> where TStorage : VoxelStorage<Voxel>
{
    device: Arc<wgpu::Device>,
    
    camera: Camera,
    camera_uniform: RefCell<Uniform<CameraUniform>>,
    _voxel_size_uniform: Uniform<f32>,
    _voxel_color_storage: Uniform<[Color; 4]>,
    chunk_position_uniform: RefCell<Uniform<GPUVec4<i32>>>,

    vertex_buffer: VertexBuffer<VoxelVertex>,
    index_buffer: IndexBuffer,

    terrain: Arc<Mutex<VoxelTerrain<TStorage>>>,
    terrain_bind_group: BindGroup,

    render_pipeline: wgpu::RenderPipeline,
}

impl<TStorage> TerrainRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    pub fn new(terrain: Arc<Mutex<VoxelTerrain<TStorage>>>, camera: Camera, device: Arc<wgpu::Device>, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let terrain_mutex = terrain.lock().unwrap();

        let mut camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);

        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, &device);
        let voxel_size_uniform = Uniform::new(terrain_mutex.info().voxel_size, wgpu::ShaderStages::VERTEX, &device);

        let chunk_position_uniform = Uniform::new(GPUVec4::new(0, 0, 0, 0), wgpu::ShaderStages::VERTEX, &device);

        let voxel_colors: [Color; 4] = terrain_mutex
            .info().voxel_types
            .iter()
            .map(|v| v.color.into())
            .collect::<Vec<_>>().try_into().unwrap();

        let voxel_color_storage = Uniform::new(voxel_colors, wgpu::ShaderStages::VERTEX, &device);

        let vertex_buffer = VertexBuffer::new(&VOXEL_FACE_VERTICES, &device, Some("Voxel Vertex Buffer"));
        let index_buffer = IndexBuffer::new(&VOXEL_FACE_TRIANGLES, &device, Some("Voxel Index Buffer"));

        let terrain_bind_group = BindGroup::new(&[&camera_uniform, &voxel_size_uniform, &chunk_position_uniform, &voxel_color_storage], &device);

        println!("Camera uniform size {}", camera_uniform.size());
        println!("Voxel size uniform size {}", voxel_size_uniform.size());
        println!("Chunk position uniform size {}", chunk_position_uniform.size());
        println!("Voxel color uniform size {}", voxel_color_storage.size());

        let shader = &device.create_shader_module(wgpu::include_spirv!(env!("terrain_shader.spv")));
        let render_pipeline = construct_render_pipeline(&device, config, &RenderPipelineInfo {
            shader,
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[&VoxelFace::desc(), &VoxelVertex::desc()],
            bind_groups: &[terrain_bind_group.layout()],
            label: Some("Voxel Render Pipeline")
        });

        drop(terrain_mutex);

        Self 
        { 
            device, 
            camera, 
            camera_uniform: RefCell::new(camera_uniform), 
            _voxel_size_uniform: voxel_size_uniform, 
            _voxel_color_storage: voxel_color_storage, 
            chunk_position_uniform: RefCell::new(chunk_position_uniform),
            vertex_buffer,
            index_buffer,
            terrain_bind_group, 
            terrain, 
            render_pipeline 
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }
}

impl<TStorage> RenderStage for TerrainRenderStage<TStorage> 
    where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn on_draw(&self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, depth_texture: &crate::gpu_utils::Texture) 
    {
        let terrain = self.terrain.lock().unwrap();
        for chunk in terrain.chunks()
        {
            let Some(render_data) = chunk.render_data() else { continue; };

            // update camera view
            let mut data = CameraUniform::new();
            data.update_view_proj(&self.camera);
            self.camera_uniform.borrow_mut().enqueue_write(data, queue);

            // update chunk position
            let chunk_index: Vec3<i32> = chunk.index().cast().unwrap();
            let chunk_position = (chunk_index * terrain.info().chunk_length() as i32).extend(0);
            self.chunk_position_uniform.borrow_mut().enqueue_write(chunk_position.into(), queue);

            let mut command_encoder = get_command_encoder(device);
            let info = RenderPassInfo
            {
                command_encoder: &mut command_encoder,
                render_pipeline: &self.render_pipeline,
                bind_groups: &[self.terrain_bind_group.bind_group()],
                view,
                depth_texture: Some(depth_texture),
                vertex_buffers: &[render_data.face_instance_buffer().slice_all(), self.vertex_buffer.slice_all()],
                index_buffer: Some(self.index_buffer.slice(..)),
                index_format: wgpu::IndexFormat::Uint32,
            };

            let mut render_pass = build_render_pass(info);
            render_pass.draw_indexed(0..6, 0, 0..(render_data.face_instance_buffer().length() as u32));
            drop(render_pass);

            queue.submit(std::iter::once(command_encoder.finish()));
        }
    }
}
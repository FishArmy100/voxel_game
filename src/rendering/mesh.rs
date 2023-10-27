use std::cell::RefCell;

use crate::camera::{Camera, CameraUniform};
use crate::math::*;
use crate::rendering::RenderStage;

use crate::gpu_utils::{BindGroup, Uniform, VertexBuffer, VertexData, IndexBuffer, Texture};
use super::{construct_render_pipeline, RenderPipelineInfo, get_command_encoder, RenderPassInfo, build_render_pass};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex 
{
    position: Vec3<f32>,
    color: Color
}

impl Vertex
{
    pub fn new(position: Vec3<f32>, color: Color) -> Self
    {
        Self { position, color }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl VertexData for Vertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle(u32, u32, u32);

unsafe impl bytemuck::Pod for Triangle {}
unsafe impl bytemuck::Zeroable for Triangle {}

pub struct Mesh 
{
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>
}

impl Mesh 
{
    pub fn new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Self
    {
        Self { vertices, triangles }
    }

    pub fn get_triangle_indexes(&self) -> &[u32]
    {
        bytemuck::cast_slice(&self.triangles)
    }

    pub fn cube(color: Color) -> Self
    {
        let vertices = vec![
            Vertex::new(Vec3::new(0., 0., 0.), color),
            Vertex::new(Vec3::new(1., 0., 0.), color),
            Vertex::new(Vec3::new(0., 1., 0.), color),
            Vertex::new(Vec3::new(1., 1., 0.), color),
            Vertex::new(Vec3::new(0., 0., 1.), color),
            Vertex::new(Vec3::new(1., 0., 1.), color),
            Vertex::new(Vec3::new(0., 1., 1.), color),
            Vertex::new(Vec3::new(1., 1., 1.), color),
        ];

        let triangles = vec![
            // Top
            Triangle(2, 6, 7),
            Triangle(7, 3, 2),

            //Bottom
            Triangle(5, 4, 0),
            Triangle(0, 1, 5),

            //Left
            Triangle(6, 2, 0),
            Triangle(0, 4, 6),

            //Right
            Triangle(1, 3, 7),
            Triangle(7, 5, 1),

            //Front
            Triangle(0, 2, 3),
            Triangle(3, 1, 0),

            //Back
            Triangle(7, 6, 4),
            Triangle(4, 5, 7)
        ];

        Mesh::new(vertices, triangles)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshInstance
{
    data_raw: [[f32; 4]; 4]
}

impl MeshInstance
{
    pub fn from_position(position: Vec3<f32>) -> Self
    {
        let mat = Mat4x4::from_translation(position);
        Self 
        {
            data_raw: mat.into() 
        }
    }
}

unsafe impl bytemuck::Pod for MeshInstance {}
unsafe impl bytemuck::Zeroable for MeshInstance {}

impl VertexData for MeshInstance
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct MeshRenderStage
{
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    instance_buffer: VertexBuffer<MeshInstance>,
    render_pipeline: wgpu::RenderPipeline,

    camera_uniform: RefCell<Uniform<CameraUniform>>,
    camera_bind_group: BindGroup,
    camera: Camera
}

impl MeshRenderStage
{
    pub fn new(mesh: Mesh, transforms: &[MeshInstance], camera: Camera, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let vertex_buffer = VertexBuffer::new(&mesh.vertices, device, None);
        let index_buffer = IndexBuffer::new(mesh.get_triangle_indexes(), device, None);
        let instance_buffer = VertexBuffer::new(transforms, device, None);

        let mut camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);
        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, device);

        let camera_bind_group = BindGroup::new(&[&camera_uniform], device);

        let shader = &device.create_shader_module(wgpu::include_wgsl!("../shaders/mesh_shader.wgsl"));
        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo 
        { 
            shader,
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[&Vertex::desc(), &MeshInstance::desc()],
            bind_groups: &[camera_bind_group.layout()], 
            label: Some("Mesh render pipeline")
        });

        Self 
        { 
            vertex_buffer, 
            index_buffer, 
            instance_buffer, 
            render_pipeline,
            camera_uniform: RefCell::new(camera_uniform), 
            camera_bind_group, 
            camera 
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera
    }
}

impl RenderStage for MeshRenderStage
{
    fn on_draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, depth_texture: &Texture) 
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_write(camera_uniform, queue);

        let mut command_encoder = get_command_encoder(device);
        let info = RenderPassInfo
        {
            command_encoder: &mut command_encoder,
            render_pipeline: &self.render_pipeline,
            bind_groups: &[self.camera_bind_group.bind_group()],
            view,
            depth_texture: Some(depth_texture),
            vertex_buffers: &[self.vertex_buffer.slice_all(), self.instance_buffer.slice_all()],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint32,
        };

        let mut render_pass = build_render_pass(info);
        render_pass.draw_indexed(0..(self.index_buffer.capacity() as u32), 0, 0..(self.instance_buffer.capacity() as u32));
        drop(render_pass);

        queue.submit(std::iter::once(command_encoder.finish()));
    }
}


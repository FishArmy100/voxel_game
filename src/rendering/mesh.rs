use crate::camera::{Camera, CameraUniform};
use crate::math::*;
use crate::colors::Color;
use crate::rendering::{VertexData, RenderStage, DrawCall};

use super::bind_group::{BindGroup, Uniform};
use super::{VertexBuffer, IndexBuffer, construct_render_pipeline, RenderPipelineInfo};

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

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]).iter());
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

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]).iter());
    }
}

pub struct MeshRenderStage
{
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer,
    instance_buffer: VertexBuffer<MeshInstance>,
    render_pipeline: wgpu::RenderPipeline,

    camera_uniform: Uniform<CameraUniform>,
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

        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo 
        { 
            shader_source: include_str!("../shaders/mesh_shader.wgsl"), 
            shader_name: Some("Mesh Shader"),
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[vertex_buffer.layout(), instance_buffer.layout()],
            bind_groups: &[camera_bind_group.layout()], 
            label: Some("Mesh render pipeline")
        });

        Self 
        { 
            vertex_buffer, 
            index_buffer, 
            instance_buffer, 
            render_pipeline,
            camera_uniform, 
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
    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> {
        vec![Box::new(MeshDrawCall {
            vertex_buffer: &self.vertex_buffer,
            index_buffer: &self.index_buffer,
            instance_buffer: &self.instance_buffer,
            camera_uniform: &self.camera_uniform,
            camera_bind_group: &self.camera_bind_group,
            camera: self.camera.clone()
        })]
    }
}

pub struct MeshDrawCall<'b>
{
    vertex_buffer: &'b VertexBuffer<Vertex>,
    index_buffer: &'b IndexBuffer,
    instance_buffer: &'b VertexBuffer<MeshInstance>,
    camera_uniform: &'b Uniform<CameraUniform>,
    camera_bind_group: &'b BindGroup,
    camera: Camera
}

impl<'buffer> DrawCall for MeshDrawCall<'buffer>
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([&self.camera_bind_group])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_uniform.enqueue_set(camera_uniform, queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..(self.index_buffer.capacity as u32), 0, 0..(self.instance_buffer.capacity() as u32));
    }
}


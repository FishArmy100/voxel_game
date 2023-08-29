pub mod renderer;
pub mod voxel_render_stage;
pub mod debug_render_stage;
pub mod mesh;

use std::sync::Arc;

use crate::{math::{Vec3, Mat4x4, Point3D}, voxel::terrain::VoxelTerrain, camera::Camera, colors::Color};
use wgpu::{util::DeviceExt};

use self::{renderer::Renderer, debug_render_stage::{DebugRenderStage, DebugLine, DebugObject}, voxel_render_stage::VoxelRenderStage};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelUniform
{
    pub model: Mat4x4<f32>
}

impl ModelUniform
{
    pub fn new(mat: Mat4x4<f32>) -> Self
    {
        Self { model: mat }
    }

    pub fn from_position(position: Point3D<f32>) -> Self
    {
        let mat = Mat4x4::from_translation(Vec3::new(position.x, position.y, position.z));
        Self::new(mat)
    }
}

unsafe impl bytemuck::Pod for ModelUniform {}
unsafe impl bytemuck::Zeroable for ModelUniform {}

pub struct BindGroupData 
{
    name: String,
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl BindGroupData
{
    pub fn name(&self) -> &str { &self.name }
    pub fn layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
    pub fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }

    pub fn uniform<T>(name: String, data: T, shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        let layout = Self::get_uniform_layout(shader_stages, device);
        Self::uniform_with_layout(name, data, layout, device)
    }

    pub fn uniform_bytes(name: String, data: &[u8], shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> Self
    {
        let layout = Self::get_uniform_layout(shader_stages, device);
        Self::uniform_with_layout_bytes(name, data, layout, device)
    }

    pub fn uniform_with_layout<T>(name: String, data: T, layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        let data_array = &[data];
        let data: &[u8] = bytemuck::cast_slice(data_array);

        let (buffer, bind_group) = Self::get_bind_group(&layout, data, device);
        Self { name, layout, buffer, bind_group }
    }

    pub fn uniform_with_layout_bytes(name: String, data: &[u8], layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self
    {
        let (buffer, bind_group) = Self::get_bind_group(&layout, data, device);
        Self { name, layout, buffer, bind_group }
    }

    pub fn get_uniform_layout(shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: shader_stages,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        })
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, data: T) 
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[data]));
    }

    fn get_bind_group(layout: &wgpu::BindGroupLayout, data: &[u8], device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroup)
    {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: None,
        });

        (buffer, bind_group)
    }
}

pub trait RenderStage
{
    fn bind_groups(&self) -> &[BindGroupData];
    fn render_pipeline(&self) -> &wgpu::RenderPipeline;
    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>;
}

pub trait DrawCall
{
    fn on_pre_draw(&self, queue: &wgpu::Queue);
    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>);
}

pub trait VertexData
{
    fn desc() -> wgpu::VertexBufferLayout<'static>;
    fn append_bytes(&self, bytes: &mut Vec<u8>);
}

fn collect_bytes_from_vertex_slice<T>(vertices: &[T]) -> Vec<u8>
    where T : VertexData
{
    let mut bytes = Vec::with_capacity(vertices.len() * T::desc().array_stride as usize);

    for vert in vertices
    {
        vert.append_bytes(&mut bytes);
    }

    bytes
}

pub struct VertexBuffer
{
    buffer: wgpu::Buffer,
    capacity: u64,
    layout: wgpu::VertexBufferLayout<'static>
}

impl VertexBuffer
{
    pub fn capacity(&self) -> u64 { self.capacity }
    pub fn layout(&self) -> &wgpu::VertexBufferLayout<'static> { &self.layout }

    pub fn new<T>(vertices: &[T], device: &wgpu::Device, label: Option<&str>) -> Self
        where T : VertexData
    {
        let layout = T::desc();
        let capacity = vertices.len() as u64;
        let data = collect_bytes_from_vertex_slice(vertices);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity, layout }
    }

    pub fn new_empty<T>(device: &wgpu::Device, capacity: u64, label: Option<&str>) -> Self 
        where T : VertexData
    {
        let layout = T::desc();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &vec![0 as u8; (layout.array_stride * capacity) as usize],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity, layout }
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, vertices: &[T])
        where T : VertexData
    {
        assert!(vertices.len() as u64 <= self.capacity, "Data is larger than the capacity of this buffer.");
        assert!(T::desc() == self.layout, "Layout for the data is different than the layout of this buffer.");

        let data = collect_bytes_from_vertex_slice(vertices);
        queue.write_buffer(&self.buffer, 0, &data)
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice
        where B : std::ops::RangeBounds<wgpu::BufferAddress>
    {
        self.buffer.slice(bounds)
    }
}

pub struct IndexBuffer 
{
    buffer: wgpu::Buffer,
    capacity: u64
}

impl IndexBuffer
{
    pub fn capacity(&self) -> u64 { self.capacity }

    pub fn new(device: &wgpu::Device, indices: &[u16], label: Option<&str>) -> Self
    {
        let capacity = indices.len() as u64;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn new_empty(device: &wgpu::Device, capacity: u64, label: Option<&str>) -> Self
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &vec![0 as u8; capacity as usize * 2],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, indices: &[u16])
        where T : VertexData
    {
        assert!(indices.len() as u64 <= self.capacity, "Data is larger than the capacity of this buffer.");

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(indices));
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice
        where B : std::ops::RangeBounds<wgpu::BufferAddress>
    {
        self.buffer.slice(bounds)
    }
}

pub struct GameRenderer
{
    renderer: Renderer,
    voxel_stage: VoxelRenderStage,
    debug_stage: DebugRenderStage
}

impl GameRenderer
{
    pub fn new(terrain: Arc<VoxelTerrain>, camera: Camera, device: Arc<wgpu::Device>, surface: Arc<wgpu::Surface>, queue: Arc<wgpu::Queue>, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let clear_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let renderer = Renderer::new(device.clone(), surface, queue, config, clear_color);

        let voxel_stage = VoxelRenderStage::new(terrain, camera.clone(), &device, config);
        let debug_stage = DebugRenderStage::new(device.clone(), config, camera.clone(), &[]);

        Self { renderer, voxel_stage, debug_stage }
    }

    pub fn update(&mut self, camera: &Camera, debug_objects: &[DebugObject])
    {
        self.voxel_stage.update(camera.clone());
        self.debug_stage.update(debug_objects, camera.clone());
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError>
    {
        self.renderer.render(&[&self.voxel_stage, &self.debug_stage])
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration)
    {
        self.renderer.resize(config);
    }
}
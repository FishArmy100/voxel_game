pub mod renderer;
pub mod debug_rendering;
pub mod mesh;

use std::{sync::{Arc, Mutex}, marker::PhantomData, ops::RangeBounds};

use crate::{math::{Vec3, Mat4x4, Point3D}, voxel::{terrain::VoxelTerrain, VoxelStorage, Voxel, terrain_renderer::{VoxelMesh, FaceDir, VoxelRenderStage}, brick_map::{BrickMap, SizedBrickMap}}, camera::Camera, colors::Color, texture::Texture, utils::Byteable, gpu_utils::bind_group::BindGroup};
use cgmath::Array;
use wgpu::{util::DeviceExt, VertexBufferLayout, BindGroupLayout};

use self::{renderer::Renderer, debug_rendering::{DebugRenderStage, DebugLine, DebugObject}, mesh::{MeshRenderStage, Mesh, MeshInstance}};

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

pub trait RenderStage
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline;
    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>;
}

pub trait DrawCall
{
    fn bind_groups(&self) -> Box<[&BindGroup]>;
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

pub trait IVertexBuffer
{
    fn capacity(&self) -> u64;
    fn layout(&self) -> &wgpu::VertexBufferLayout<'static>;
    fn slice(&self, first: wgpu::BufferAddress, last: wgpu::BufferAddress) -> wgpu::BufferSlice;
    fn slice_all(&self) -> wgpu::BufferSlice;
}

pub struct VertexBuffer<T> where T : VertexData
{
    buffer: wgpu::Buffer,
    capacity: u64,
    layout: wgpu::VertexBufferLayout<'static>,
    phantom: PhantomData<T>
}

impl<T> VertexBuffer<T> where T : VertexData
{
    pub fn capacity(&self) -> u64 { self.capacity }
    pub fn layout(&self) -> &wgpu::VertexBufferLayout<'static> { &self.layout }

    pub fn new(vertices: &[T], device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let layout = T::desc();
        let capacity = vertices.len() as u64;
        let data = collect_bytes_from_vertex_slice(vertices);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity, layout, phantom: PhantomData }
    }

    pub fn new_empty(device: &wgpu::Device, capacity: u64, label: Option<&str>) -> Self
    {
        let layout = T::desc();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &vec![0 as u8; (layout.array_stride * capacity) as usize],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity, layout, phantom: PhantomData }
    }

    pub fn enqueue_set_data(&self, queue: &wgpu::Queue, vertices: &[T])
    {
        assert!(vertices.len() as u64 <= self.capacity, "Data is larger than the capacity of this buffer.");
        assert!(T::desc() == self.layout, "Layout for the data is different than the layout of this buffer.");

        let data = collect_bytes_from_vertex_slice(vertices);
        queue.write_buffer(&self.buffer, 0, &data)
    }

    fn slice(&self, first: wgpu::BufferAddress, last: wgpu::BufferAddress) -> wgpu::BufferSlice
    {
        self.buffer.slice(first..last)
    }

    fn slice_all(&self) -> wgpu::BufferSlice
    {
        self.buffer.slice(..)
    }
}

impl<T> IVertexBuffer for VertexBuffer<T> where T : VertexData
{
    fn capacity(&self) -> u64 { self.capacity() }
    fn layout(&self) -> &wgpu::VertexBufferLayout<'static> { self.layout() }

    fn slice(&self, first: wgpu::BufferAddress, last: wgpu::BufferAddress) -> wgpu::BufferSlice
    {
        self.slice(first, last)
    }

    fn slice_all(&self) -> wgpu::BufferSlice 
    {
        self.slice_all()
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

    pub fn new(indices: &[u32], device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let capacity = indices.len() as u64;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn new_empty(capacity: u64, device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &vec![0 as u8; capacity as usize * std::mem::size_of::<u32>()],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, indices: &[u32])
        where T : VertexData
    {
        assert!(indices.len() as u64 <= self.capacity, "Data is larger than the capacity of this buffer.");

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(indices));
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice
        where B : RangeBounds<wgpu::BufferAddress>
    {
        self.buffer.slice(bounds)
    }
}

pub struct RenderPipelineInfo<'l>
{
    pub shader_source: &'l str,
    pub shader_name: Option<&'l str>,

    pub vs_main: &'l str,
    pub fs_main: &'l str,

    pub vertex_buffers: &'l [&'l VertexBufferLayout<'l>],
    pub bind_groups: &'l [&'l BindGroupLayout],

    pub label: Option<&'l str>
}

pub fn construct_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, info: &RenderPipelineInfo) -> wgpu::RenderPipeline
{
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(info.shader_source.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &info.bind_groups,
        push_constant_ranges: &[]
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: info.label,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: info.vs_main,
            buffers: &info.vertex_buffers.iter()
                .map(|b| (*b).clone())
                .collect::<Vec<_>>()
        },
        
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: info.fs_main,
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL
            })],
        }),

        primitive: wgpu::PrimitiveState { 
            topology: wgpu::PrimitiveTopology::TriangleList, 
            strip_index_format: None, 
            front_face: wgpu::FrontFace::Ccw, 
            cull_mode: Some(wgpu::Face::Back), 
            unclipped_depth: false, 
            polygon_mode: wgpu::PolygonMode::Fill, 
            conservative: false 
        },

        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // 1.
            stencil: wgpu::StencilState::default(), // 2.
            bias: wgpu::DepthBiasState::default(),
        }),
    
        multisample: wgpu::MultisampleState { 
            count: 1, 
            mask: !0, 
            alpha_to_coverage_enabled: false 
        },
        multiview: None
    });

    render_pipeline
}

pub struct GameRenderer
{
    renderer: Renderer,
    debug_stage: DebugRenderStage,
    mesh_stage: MeshRenderStage,
    voxel_stage: VoxelRenderStage,
}

impl GameRenderer
{
    pub fn new<TStorage>(terrain: Arc<Mutex<VoxelTerrain<TStorage>>>, camera: Camera, device: Arc<wgpu::Device>, surface: Arc<wgpu::Surface>, queue: Arc<wgpu::Queue>, config: &wgpu::SurfaceConfiguration) -> Self
        where TStorage : VoxelStorage<Voxel> + Send + 'static
    {
        let clear_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let renderer = Renderer::new(device.clone(), surface, queue, config, clear_color);

        let debug_stage = DebugRenderStage::new(device.clone(), config, camera.clone(), &[]);
        let mesh_stage = MeshRenderStage::new(Mesh::cube(Color::RED), &[MeshInstance::from_position([0.0, 2.0, 0.0].into())], camera.clone(), &device, config);
        
        let terrain = terrain.lock().unwrap();
        let mesh = terrain.chunks()[0].storage().get_mesh(); 

        let mut voxel_stage = VoxelRenderStage::new(camera.clone(), device.clone(), config);
        voxel_stage.add_mesh(&mesh, terrain.info().voxel_size, Vec3::from_value(0));

        Self 
        { 
            renderer, 
            debug_stage, 
            mesh_stage, 
            voxel_stage,
        }
    }

    pub fn update(&mut self, camera: &Camera, debug_objects: &[DebugObject])
    {
        self.debug_stage.update(debug_objects, camera.clone());
        self.mesh_stage.update(camera.clone());
        self.voxel_stage.update(camera.clone());
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
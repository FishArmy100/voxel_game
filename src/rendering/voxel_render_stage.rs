use std::sync::Arc;

use cgmath::EuclideanSpace;
use wgpu::util::DeviceExt;

use crate::camera::{Camera, CameraUniform, self};
use crate::debug_utils;
use crate::math::{Vec3, Mat4x4, Point3D};
use crate::rendering::ModelUniform;
use crate::texture::Texture;
use crate::voxel::{VoxelData, VoxelTerrain};

use crate::colors::Color;
use super::{RenderStage, DrawCall, BindGroupData};

pub const VOXEL_FACE_VERTICES: [VoxelVertex; 4] = [VoxelVertex::new(0, Color::WHITE), VoxelVertex::new(1, Color::RED), VoxelVertex::new(2, Color::GREEN), VoxelVertex::new(3, Color::BLUE)];
pub const VOXEL_FACE_TRIANGLES: [u16; 6] = [2, 1, 0, 2, 3, 1];
pub struct VoxelFaces();

impl VoxelFaces
{
    pub const UP: u32 = 0;
    pub const DOWN: u32 = 1;
    pub const NORTH: u32 = 2;
    pub const SOUTH: u32 = 3;
    pub const EAST: u32 = 4;
    pub const WEST: u32 = 5;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex 
{
    pub index: u32,
    pub color: Color
}

impl VoxelVertex
{
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Uint32, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub const fn new(index: u32, color: Color) -> Self
    {
        Self { index, color }
    }
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelFaceData
{
    pub position: Vec3<u32>,
    pub id: u32,
    pub face_index: u32
}

impl VoxelFaceData
{
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![2 => Uint32x3, 3 => Uint32, 4 => Uint32];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(position: Vec3<u32>, id: u32, face_index: u32) -> Self
    {
        Self { position, id, face_index }
    }
}

unsafe impl bytemuck::Pod for VoxelFaceData {}
unsafe impl bytemuck::Zeroable for VoxelFaceData {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelRenderData 
{
    pub color: Color
}

impl VoxelRenderData 
{
    pub fn new(color: Color) -> Self 
    {
        Self { color }
    }
}

unsafe impl bytemuck::Pod for VoxelRenderData {}
unsafe impl bytemuck::Zeroable for VoxelRenderData {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VoxelRenderDataUniform
{
    pub data: Box<[VoxelRenderData]>
}

impl VoxelRenderDataUniform
{
    pub fn new(data: Box<[VoxelRenderData]>) -> Self
    {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8]
    {
        bytemuck::cast_slice(&self.data)
    }
}

pub struct VoxelRenderStage
{
    terrain: Arc<VoxelTerrain>,
    bind_groups: [BindGroupData; 3],
    render_pipeline: wgpu::RenderPipeline,

    camera: Camera,

    faces_buffer: wgpu::Buffer,
    face_buffer_capacity: u32,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}

impl VoxelRenderStage
{
    pub fn new(terrain: Arc<VoxelTerrain>, camera: Camera, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let voxel_uniform = VoxelRenderDataUniform::new(terrain.voxel_types().iter().map(|v| v.get_render_data()).collect());
        let voxel_bind_group = BindGroupData::uniform_bytes("voxel_bind_group".into(), voxel_uniform.as_bytes(), wgpu::ShaderStages::VERTEX, device);

        let model_uniform = ModelUniform::from_position(terrain.position());
        let model_bind_group = BindGroupData::uniform("model_bind_group".into(), model_uniform, wgpu::ShaderStages::VERTEX, device);

        let render_pipeline = debug_utils::time_call(|| Self::gen_render_pipeline(device, config, &camera_bind_group, &voxel_bind_group, &model_bind_group), "Constructing Render Pipeline") ;

        let vertex_buffer = Self::get_voxel_vertex_buffer(device);
        let index_buffer = Self::get_voxel_index_buffer(device);

        const FACE_BUFFER_CAPACITY: u32 = 65545;
        let faces_buffer = Self::get_faces_buffer(device, FACE_BUFFER_CAPACITY);

        Self 
        {
            terrain, 
            bind_groups: [camera_bind_group, model_bind_group, voxel_bind_group], 
            render_pipeline,
            camera,
            faces_buffer,
            face_buffer_capacity: FACE_BUFFER_CAPACITY,
            vertex_buffer,
            index_buffer
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }

    fn get_voxel_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VOXEL_FACE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    fn get_voxel_index_buffer(device: &wgpu::Device) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&VOXEL_FACE_TRIANGLES),
                usage: wgpu::BufferUsages::INDEX,
            })
    }

    fn get_faces_buffer(device: &wgpu::Device, face_buffer_capacity: u32) -> wgpu::Buffer
    {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            contents: &vec![0 as u8; std::mem::size_of::<VoxelFaceData>() * face_buffer_capacity as usize]
        })
    }

    fn gen_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group: &BindGroupData, voxel_bind_group: &BindGroupData, model_bind_group: &BindGroupData) -> wgpu::RenderPipeline
    {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/voxel_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voxel Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group.layout(), &model_bind_group.layout(), &voxel_bind_group.layout()],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VoxelVertex::desc(), VoxelFaceData::desc()]
            },
            
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
}

impl RenderStage for VoxelRenderStage
{
    fn bind_groups(&self) -> &[BindGroupData]
    {
        &self.bind_groups
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>
    {
        let faces_count = self.terrain.faces().len();
        let mut ranges = vec![self.face_buffer_capacity; faces_count / self.face_buffer_capacity as usize];
        let remainder = faces_count % self.face_buffer_capacity as usize;
        if remainder != 0 { ranges.push(remainder as u32); }

        let mut current_index: usize = 0;
        let mut draw_calls: Vec<Box<dyn DrawCall>> = vec![];
        for range in ranges
        {
            let old = current_index;
            current_index += range as usize;
            let slice = &self.terrain.faces()[old..current_index];
            let draw_call = VoxelDrawCall
            {
                voxels: slice,
                faces_buffer: &self.faces_buffer,
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                faces_length: self.terrain.faces().len() as u64,
                camera: self.camera.clone(),
                position: self.terrain.position(),
                camera_bind_group: &self.bind_groups[0],
                model_bind_group: &self.bind_groups[1]
            };

            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct VoxelDrawCall<'vox, 'buffer, 'bind_group>
{
    voxels: &'vox [VoxelFaceData],
    faces_buffer: &'buffer wgpu::Buffer,
    vertex_buffer: &'buffer wgpu::Buffer,
    index_buffer: &'buffer wgpu::Buffer,
    faces_length: u64,

    camera: Camera,
    position: Point3D<f32>,

    camera_bind_group: &'bind_group BindGroupData,
    model_bind_group: &'bind_group BindGroupData
}

impl<'vox, 'buffer, 'bind_group> DrawCall for VoxelDrawCall<'vox, 'buffer, 'bind_group>
{
    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        queue.write_buffer(&self.faces_buffer, 0, bytemuck::cast_slice(self.voxels));

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_bind_group.enqueue_set_data(queue, camera_uniform);

        let model_uniform = ModelUniform::from_position(self.position);
        self.model_bind_group.enqueue_set_data(queue, model_uniform);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>)
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.faces_buffer.slice(0..((self.faces_length as usize * std::mem::size_of::<VoxelFaceData>()) as u64)));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..6, 0, 0..(self.faces_length as u32));
    }
}
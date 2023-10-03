use std::sync::Arc;
use std::cell::RefCell;

use cgmath::{Zero, ElementWise};
use wgpu::util::DeviceExt;

use super::bind_group::{Uniform, BindGroup};
use super::{RenderStage, DrawCall};
use crate::camera::{Camera, CameraUniform};
use crate::math::Vec3;
use crate::colors::Color;
use crate::texture::Texture;

#[derive(Debug, Clone, Copy)]
pub struct DebugLine
{
    pub a: Vec3<f32>,
    pub b: Vec3<f32>,
    pub color: Color
}

impl DebugLine
{
    pub fn new(a: Vec3<f32>, b: Vec3<f32>, color: Color) -> Self
    {
        DebugLine { a, b, color }
    }

    fn append_vertices(&self, vec: &mut Vec<DebugLineVertex>)
    {
        vec.push(DebugLineVertex::new(self.a, self.color));
        vec.push(DebugLineVertex::new(self.b, self.color));
    }
}

pub struct DebugCube
{
    pub position: Vec3<f32>,
    pub extents: Vec3<f32>,
    pub color: Color
}

impl DebugCube
{
    pub fn new(position: Vec3<f32>, extents: Vec3<f32>, color: Color) -> Self
    {
        Self { position, extents, color }
    }

    fn append_vertices(&self, vec: &mut Vec<DebugLineVertex>)
    {
        let base_a = DebugLine
        {
            a: self.position + Vec3::new(0.0, 0.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 0.0, 0.0).mul_element_wise(self.extents),
            color: self.color
        };

        let base_b = DebugLine
        {
            a: self.position + Vec3::new(0.0, 0.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(0.0, 0.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let base_c = DebugLine
        {
            a: self.position + Vec3::new(0.0, 0.0, 1.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 0.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let base_d = DebugLine
        {
            a: self.position + Vec3::new(1.0, 0.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 0.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };


        let top_a = DebugLine
        {
            a: self.position + Vec3::new(0.0, 1.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 1.0, 0.0).mul_element_wise(self.extents),
            color: self.color
        };

        let top_b = DebugLine
        {
            a: self.position + Vec3::new(0.0, 1.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(0.0, 1.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let top_c = DebugLine
        {
            a: self.position + Vec3::new(0.0, 1.0, 1.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 1.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let top_d = DebugLine
        {
            a: self.position + Vec3::new(1.0, 1.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 1.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let middle_a = DebugLine
        {
            a: self.position + Vec3::new(0.0, 0.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(0.0, 1.0, 0.0).mul_element_wise(self.extents),
            color: self.color
        };

        let middle_b = DebugLine
        {
            a: self.position + Vec3::new(1.0, 0.0, 0.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 1.0, 0.0).mul_element_wise(self.extents),
            color: self.color
        };

        let middle_c = DebugLine
        {
            a: self.position + Vec3::new(0.0, 0.0, 1.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(0.0, 1.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        let middle_d = DebugLine
        {
            a: self.position + Vec3::new(1.0, 0.0, 1.0).mul_element_wise(self.extents),
            b: self.position + Vec3::new(1.0, 1.0, 1.0).mul_element_wise(self.extents),
            color: self.color
        };

        base_a.append_vertices(vec);
        base_b.append_vertices(vec);
        base_c.append_vertices(vec);
        base_d.append_vertices(vec);

        top_a.append_vertices(vec);
        top_b.append_vertices(vec);
        top_c.append_vertices(vec);
        top_d.append_vertices(vec);

        middle_a.append_vertices(vec);
        middle_b.append_vertices(vec);
        middle_c.append_vertices(vec);
        middle_d.append_vertices(vec);
    }
}

pub enum DebugObject
{
    Line(DebugLine),
    Cube(DebugCube)
}

impl DebugObject
{
    fn append_vertices(&self, vec: &mut Vec<DebugLineVertex>)
    {
        match self 
        {
            Self::Line(l) => l.append_vertices(vec),
            Self::Cube(c) => c.append_vertices(vec)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DebugLineVertex
{
    pub position: Vec3<f32>,
    pub color: Color
}

impl DebugLineVertex
{
    pub fn new(position: Vec3<f32>, color: Color) -> Self
    {
        Self { position, color }
    }

    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

unsafe impl bytemuck::Pod for DebugLineVertex {}
unsafe impl bytemuck::Zeroable for DebugLineVertex {}

pub struct DebugRenderStage
{
    device: Arc<wgpu::Device>,

    render_pipeline: wgpu::RenderPipeline,

    camera_uniform: RefCell<Uniform<CameraUniform>>,
    bind_group: BindGroup,

    camera: Camera,

    vertex_buffer: wgpu::Buffer,
    vertex_count: u32
}

impl DebugRenderStage
{
    pub fn new(device: Arc<wgpu::Device>, config: &wgpu::SurfaceConfiguration, default_camera: Camera, debug_objects: &[DebugObject]) -> Self
    {
        let camera_uniform = Uniform::<CameraUniform>::new_empty(wgpu::ShaderStages::VERTEX, &device);
        let bind_group = BindGroup::new(&[&camera_uniform], &device);

        let render_pipeline = Self::gen_render_pipeline(&device, config, &bind_group);

        let (vertex_buffer, vertex_count) = Self::get_vertex_buffer(&device, debug_objects);

        Self 
        { 
            device: device.clone(), 
            render_pipeline, 
            camera_uniform: RefCell::new(camera_uniform),
            bind_group, 
            camera: default_camera, 
            vertex_buffer, 
            vertex_count
        }
    }

    pub fn update(&mut self, debug_objects: &[DebugObject], camera: Camera)
    {
        let (vertex_buffer, vertex_count) = Self::get_vertex_buffer(&self.device, debug_objects);
        
        self.vertex_buffer = vertex_buffer;
        self.vertex_count = vertex_count;
        self.camera = camera;
    }

    fn get_vertex_buffer(device: &wgpu::Device, debug_objects: &[DebugObject]) -> (wgpu::Buffer, u32)
    {
        let mut vertices = vec![];
        for object in debug_objects
        {
            object.append_vertices(&mut vertices);
        }

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        (buffer, vertices.len() as u32)
    }

    fn gen_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group: &BindGroup) -> wgpu::RenderPipeline
    {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/debug_shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group.layout()],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DebugLineVertex::desc()]
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
                topology: wgpu::PrimitiveTopology::LineList, 
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

impl RenderStage for DebugRenderStage
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        vec![Box::new(DebugDrawCall::new(self.camera.clone(), &self.camera_uniform, &self.bind_group, &self.vertex_buffer, self.vertex_count))]
    }
}

pub struct DebugDrawCall<'b>
{
    camera: Camera,

    camera_uniform: &'b RefCell<Uniform<CameraUniform>>,
    bind_group: &'b BindGroup,

    vertex_buffer: &'b wgpu::Buffer,
    vertex_count: u32
}

impl<'b> DebugDrawCall<'b>
{
    pub fn new(camera: Camera, camera_uniform: &'b RefCell<Uniform<CameraUniform>>, bind_group: &'b BindGroup, vertex_buffer: &'b wgpu::Buffer, vertex_count: u32) -> Self
    {
        Self 
        { 
            camera, 
            camera_uniform,
            bind_group, 
            vertex_buffer, 
            vertex_count 
        }
    }
}

impl<'b> DrawCall for DebugDrawCall<'b>
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([&self.bind_group])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_set(camera_uniform, queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
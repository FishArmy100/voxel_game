use wgpu::util::DeviceExt;

use super::{RenderStage, DrawCall, BindGroupData};
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

    fn get_vertices(&self) -> [DebugLineVertex; 2]
    {
        [DebugLineVertex::new(self.a, self.color), DebugLineVertex::new(self.b, self.color)]
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
    render_pipeline: wgpu::RenderPipeline,
    bind_groups: [BindGroupData; 1],

    vertex_buffer: wgpu::Buffer,
    vertex_count: u32
}

impl DebugRenderStage
{
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera: &Camera, debug_lines: &[DebugLine]) -> Self
    {
        let vertices = debug_lines.iter()
            .map(|l| l.get_vertices())
            .fold(vec![], |mut vec, vs| 
            {
                vec.push(vs[0]); 
                vec.push(vs[1]); 
                vec
            });

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let render_pipeline = Self::gen_render_pipeline(device, config, &camera_bind_group);
        let vertex_buffer = Self::get_vertex_buffer(device, &vertices);

        Self { render_pipeline, bind_groups: [camera_bind_group], vertex_buffer, vertex_count: vertices.len() as u32 }
    }

    fn get_vertex_buffer(device: &wgpu::Device, data: &[DebugLineVertex]) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    fn gen_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera_bind_group: &BindGroupData) -> wgpu::RenderPipeline
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
    fn bind_groups(&self) -> &[super::BindGroupData] {
        &self.bind_groups
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        vec![Box::new(DebugDrawCall::new(&self.vertex_buffer, self.vertex_count))]
    }
}

pub struct DebugDrawCall<'buffer>
{
    vertex_buffer: &'buffer wgpu::Buffer,
    vertex_count: u32
}

impl<'buffer> DebugDrawCall<'buffer>
{
    pub fn new(vertex_buffer: &'buffer wgpu::Buffer, vertex_count: u32) -> Self
    {
        Self { vertex_buffer, vertex_count }
    }
}

impl<'buffer> DrawCall for DebugDrawCall<'buffer>
{
    fn on_pre_draw(&self, _queue: &wgpu::Queue) {}

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}
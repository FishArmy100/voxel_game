pub mod renderer;
pub mod gui;

use std::sync::Arc;
use crate::math::*;
use crate::gpu_utils::*;
use wgpu::{VertexBufferLayout, BindGroupLayout};

pub use crate::rendering::renderer::*;

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

pub struct RenderPipelineInfo<'a>
{
    pub shader: &'a wgpu::ShaderModule,

    pub vs_main: &'a str,
    pub fs_main: &'a str,

    pub vertex_buffers: &'a [&'a VertexBufferLayout<'a>],
    pub bind_groups: &'a [&'a BindGroupLayout],

    pub label: Option<&'a str>
}

pub fn construct_render_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, info: &RenderPipelineInfo) -> wgpu::RenderPipeline
{
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &info.bind_groups,
        push_constant_ranges: &[]
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: info.label,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &info.shader,
            entry_point: info.vs_main,
            buffers: &info.vertex_buffers.iter()
                .map(|b| (*b).clone())
                .collect::<Vec<_>>()
        },
        
        fragment: Some(wgpu::FragmentState {
            module: &info.shader,
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

pub fn get_command_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder
{
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
    { 
        label: Some("Command Encoder") 
    })
}

pub fn get_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, view: &'a wgpu::TextureView, depth_texture: Option<&'a Texture>) -> wgpu::RenderPass<'a>
{
    let depth_stencil_attachment = match depth_texture
    {
        Some(depth_texture) =>
        {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            })
        },

        None => None
    };

    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations{
                load: wgpu::LoadOp::Load,
                store: true,
            }
        })],

        depth_stencil_attachment
    })
}

pub struct RenderPassInfo<'a> 
{
    pub command_encoder: &'a mut wgpu::CommandEncoder,
    pub render_pipeline: &'a wgpu::RenderPipeline,
    pub bind_groups: &'a [&'a wgpu::BindGroup],
    pub view: &'a wgpu::TextureView,
    pub depth_texture: Option<&'a Texture>,
    pub vertex_buffers: &'a [wgpu::BufferSlice<'a>],
    pub index_buffer: Option<wgpu::BufferSlice<'a>>,
    pub index_format: wgpu::IndexFormat
}

pub fn build_render_pass<'a>(info: RenderPassInfo<'a>) -> wgpu::RenderPass<'a>
{
    let mut render_pass = get_render_pass(info.command_encoder, info.view, info.depth_texture);
    render_pass.set_pipeline(info.render_pipeline);
    for i in 0..info.bind_groups.len()
    {
        render_pass.set_bind_group(i as u32, info.bind_groups[i], &[]);
    }

    for i in 0..info.vertex_buffers.len()
    {
        render_pass.set_vertex_buffer(i as u32, info.vertex_buffers[i]);
    }

    if let Some(index_buffer) = info.index_buffer
    {
        render_pass.set_index_buffer(index_buffer, info.index_format);
    }

    render_pass
}
use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::camera::{Camera, CameraUniform};
use crate::texture::Texture;
use crate::voxel::{VoxelData, VoxelTerrain};

use super::{VoxelFaceData, VoxelRenderDataUniform, ModelUniform, VoxelVertex};
use super::renderer::{RenderStage, DrawCall, BindGroupData};


pub struct VoxelRenderStage<'terrain, const S: usize, const N: usize>
{
    terrain: &'terrain VoxelTerrain<S, N>,
    bind_groups: [BindGroupData; 3],
    render_pipeline: wgpu::RenderPipeline,

    faces_buffer: wgpu::Buffer,
    face_buffer_capacity: u32,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}

impl<'terrain, const S: usize, const N: usize> VoxelRenderStage<'terrain, S, N>
{
    pub fn new(terrain: &'terrain VoxelTerrain<S, N>, camera: &Camera, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let voxel_uniform = VoxelRenderDataUniform::new(terrain.voxel_types().map(|v| v.get_render_data()).clone());
        let voxel_bind_group = BindGroupData::uniform("voxel_bind_group".into(), voxel_uniform, wgpu::ShaderStages::VERTEX, device);

        let model_uniform = ModelUniform::from_position(terrain.position());
        let model_bind_group = BindGroupData::uniform("model_bind_group".into(), model_uniform, wgpu::ShaderStages::VERTEX, device);

        let render_pipeline = Self::gen_render_pipeline(device, config, &camera_bind_group, &voxel_bind_group, &model_bind_group);

        let vertex_buffer = Self::get_voxel_vertex_buffer(device);
        let index_buffer = Self::get_voxel_index_buffer(device);

        const FACE_BUFFER_CAPACITY: u32 = 65545;
        let faces_buffer = Self::get_faces_buffer(device, FACE_BUFFER_CAPACITY);

        Self 
        {
            terrain, 
            bind_groups: [camera_bind_group, model_bind_group, voxel_bind_group], 
            render_pipeline,
            faces_buffer,
            face_buffer_capacity: FACE_BUFFER_CAPACITY,
            vertex_buffer,
            index_buffer
        }
    }

    fn get_voxel_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&crate::rendering::VOXEL_FACE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    fn get_voxel_index_buffer(device: &wgpu::Device) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&crate::rendering::VOXEL_FACE_TRIANGLES),
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group.layout(), &model_bind_group.layout(), &voxel_bind_group.layout()],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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

impl<'terrain, const S: usize, const N: usize> RenderStage for VoxelRenderStage<'terrain, S, N>
{
    fn bind_groups(&self) -> &[super::renderer::BindGroupData] 
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
            let draw_call = VoxelDrawCall::new(slice, &self.faces_buffer, &self.vertex_buffer, &self.index_buffer, faces_count as u64);
            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct VoxelDrawCall<'vox, 'buffer>
{
    voxels: &'vox [VoxelFaceData],
    faces_buffer: &'buffer wgpu::Buffer,
    vertex_buffer: &'buffer wgpu::Buffer,
    index_buffer: &'buffer wgpu::Buffer,
    faces_length: u64
}

impl<'vox, 'buffer> VoxelDrawCall<'vox, 'buffer>
{
    pub fn new(voxels: &'vox [VoxelFaceData], faces_buffer: &'buffer wgpu::Buffer, vertex_buffer: &'buffer wgpu::Buffer, index_buffer: &'buffer wgpu::Buffer, faces_length: u64) -> Self {
        Self { voxels, faces_buffer, vertex_buffer, index_buffer, faces_length }
    }
}

impl<'vox, 'buffer> DrawCall for VoxelDrawCall<'vox, 'buffer>
{
    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        queue.write_buffer(&self.faces_buffer, 0, bytemuck::cast_slice(self.voxels));
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>)
    {
        let buffer_segment_length = (self.voxels.len() * std::mem::size_of::<VoxelFaceData>()) as u64;
        
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.faces_buffer.slice(0..((self.faces_length as usize * std::mem::size_of::<VoxelFaceData>()) as u64)));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    }
}
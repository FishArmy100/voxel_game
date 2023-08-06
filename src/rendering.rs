pub mod renderer;
pub mod voxel_render_stage;

use std::usize;
use std::sync::Arc;
use std::slice::*;

use crate::math::{Vec3, Vec2, Mat4x4, Point3D};
use crate::colors::*;
use crate::camera::{CameraUniform, Camera};
use crate::texture::Texture;
use wgpu::util::DeviceExt;

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
#[derive(Debug, Clone, Copy)]
pub struct VoxelRenderDataUniform<const S: usize>
{
    pub data: [VoxelRenderData; S]
}

impl<const S: usize> VoxelRenderDataUniform<S>
{
    pub fn new(data: [VoxelRenderData; S]) -> Self
    {
        Self { data }
    }
}

unsafe impl<const S: usize> bytemuck::Pod for VoxelRenderDataUniform<S> {}
unsafe impl<const S: usize> bytemuck::Zeroable for VoxelRenderDataUniform<S> {}

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

pub struct Renderer
{
    device: Arc<wgpu::Device>,
    surface: Arc<wgpu::Surface>,
    queue: Arc<wgpu::Queue>,

    render_pipeline: wgpu::RenderPipeline,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group_layout: wgpu::BindGroupLayout,
    render_voxel_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: Texture,

    faces_buffer: wgpu::Buffer,
    face_buffer_capacity: u32,
}

impl Renderer
{
    pub fn new(device: Arc<wgpu::Device>, surface: Arc<wgpu::Surface>, queue: Arc<wgpu::Queue>, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("model_bind_group_layout"),
        });

        let render_voxel_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("render_voxel_bind_group_layout"),
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout, &render_voxel_bind_group_layout],
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

        const FACE_BUFFER_CAPACITY: u32 = 65545;

        let faces_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: &vec![0 as u8; std::mem::size_of::<VoxelFaceData>() * FACE_BUFFER_CAPACITY as usize]
            });

        Renderer 
        { 
            device, 
            surface, 
            queue, 
            render_pipeline,
            camera_bind_group_layout,
            model_bind_group_layout,
            render_voxel_bind_group_layout,
            depth_texture,
            faces_buffer,
            face_buffer_capacity: FACE_BUFFER_CAPACITY
        }
    }

    pub fn render<const N: usize>(&mut self, camera: &Camera, faces: &[VoxelFaceData], render_voxels: VoxelRenderDataUniform<N>, transform: ModelUniform) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.clear_color(Color::new(0.1, 0.2, 0.3, 1.0), &view);
    
        let camera_bind_group = self.get_camera_bind_group(camera);
        let model_bind_group = self.get_model_bind_group(transform);
        let render_voxel_bind_group = self.get_render_voxel_bind_group(render_voxels);

        let vertex_buffer = self.get_voxel_vertex_buffer();
        let index_buffer = self.get_voxel_index_buffer();
        
        let mut ranges = vec![self.face_buffer_capacity; faces.len() / self.face_buffer_capacity as usize];
        let remainder = faces.len() % self.face_buffer_capacity as usize;
        if remainder != 0 { ranges.push(remainder as u32); }

        let mut current_face_index: usize = 0;
        for slice in &ranges
        {
            self.queue.write_buffer(&self.faces_buffer, 0, bytemuck::cast_slice(&faces[current_face_index..(current_face_index + *slice as usize)])); 

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command Encoder") });
            let mut render_pass = self.get_render_pass(&mut encoder, &view);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &model_bind_group, &[]);
            render_pass.set_bind_group(2, &render_voxel_bind_group, &[]);

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.faces_buffer.slice(0..((faces.len() * std::mem::size_of::<VoxelFaceData>()) as u64)));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..6, 0, 0..(faces.len() as u32));

            drop(render_pass);
            self.queue.submit(std::iter::once(encoder.finish())); 
            current_face_index += *slice as usize
        }
        output.present();

        Ok(())
    }

    fn clear_color(&mut self, clear_color: Color, view: &wgpu::TextureView)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("Render Encoder")
        });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations{
                    load: wgpu::LoadOp::Clear(clear_color.to_wgpu()),
                    store: true,
                }
            })],

            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn get_render_pass<'this: 'e, 'e, 't: 'e>(&'this self, encoder: &'e mut wgpu::CommandEncoder, view: &'t wgpu::TextureView) -> wgpu::RenderPass<'e>
    {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations{
                    load: wgpu::LoadOp::Load,
                    store: true,
                }
            })],

            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }

    fn get_voxel_vertex_buffer(&self) -> wgpu::Buffer
    {
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VOXEL_FACE_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            })
    }

    fn get_voxel_index_buffer(&self) -> wgpu::Buffer
    {
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&VOXEL_FACE_TRIANGLES),
                usage: wgpu::BufferUsages::INDEX,
            })
    }

    fn get_camera_bind_group(&self, camera: &Camera) -> wgpu::BindGroup
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);

        let camera_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        })
    }

    fn get_model_bind_group(&self, model: ModelUniform) -> wgpu::BindGroup
    {
        let model_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[model]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.model_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                }
            ],
            label: Some("model_bind_group"),
        })
    }

    fn get_render_voxel_bind_group<const S: usize>(&self, render_voxels: VoxelRenderDataUniform<S>) -> wgpu::BindGroup
    {
        let render_voxel_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Render Voxel Buffer"),
                contents: bytemuck::cast_slice(&[render_voxels]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.render_voxel_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_voxel_buffer.as_entire_binding(),
                }
            ],
            label: Some("render_voxel_group"),
        })
    }
}
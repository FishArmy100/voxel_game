use std::usize;

use crate::math::{Vec3, Point3D, Mat4x4};
use crate::colors::*;
use crate::camera::{CameraUniform, Camera};
use crate::texture::Texture;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex
{
    pub position: Point3D<f32>,
    pub color: Color,
}

impl Vertex
{
    pub fn new(position: Point3D<f32>, color: Color) -> Self
    {
        Self { position, color }
    }

    pub const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Clone, Copy, Debug)]
pub struct Triangle 
{
    pub indices: [u16; 3]
}

impl Triangle
{
    pub const fn new(indices: [u16; 3]) -> Self
    {
        Self { indices }
    }
}

#[derive(Debug, Clone)]
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

    pub fn get_triangles(&self) -> Vec<u16>
    {
        self.triangles.iter().map(|t| t.indices).flatten().collect()
    }

    pub fn get_vertex_buffer(device: &wgpu::Device, max_len: usize) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &vec![0 as u8; max_len * std::mem::size_of::<Vertex>()],
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    pub fn get_index_buffer(device: &wgpu::Device, max_len: usize) -> wgpu::Buffer
    {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice::<u16, u8>(&vec![0 as u16; max_len]),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModelUniform
{
    pub model: Mat4x4<f32>
}

unsafe impl bytemuck::Pod for ModelUniform {}
unsafe impl bytemuck::Zeroable for ModelUniform {}


#[derive(Clone, Debug)]
pub struct Model 
{
    pub mesh: Mesh,
    pub position: Vec3<f32>
}

impl Model
{
    pub fn new(mesh: Mesh, position: Vec3<f32>) -> Self
    {
        Self { mesh, position }
    }

    pub fn get_model_uniform(&self) -> ModelUniform
    {
        ModelUniform { model: Mat4x4::from_translation(self.position) }
    }
}

pub struct Renderer<'s, 'd, 'q, 'c, 'ms>
{
    device: &'d wgpu::Device,
    surface: &'s wgpu::Surface,
    queue: &'q mut wgpu::Queue,
    config: &'c wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group_layout: wgpu::BindGroupLayout, 
    depth_texture: Texture,

    models: Vec<&'ms Model>
}

impl<'s, 'd, 'q, 'c, 'ms> Renderer<'s, 'd, 'q, 'c, 'ms>
{
    pub fn new(device: &'d wgpu::Device, surface: &'s wgpu::Surface, queue: &'q mut wgpu::Queue, config: &'c wgpu::SurfaceConfiguration) -> Self
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

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()]
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

        Renderer 
        { 
            device, 
            surface, 
            queue, 
            models: vec![],
            config,
            render_pipeline,
            camera_bind_group_layout,
            model_bind_group_layout,
            depth_texture
        }
    }

    pub fn add_model<'m : 'ms>(&mut self, model: &'m Model)
    {
        self.models.push(model);
    }

    pub fn render(&mut self, camera: &Camera) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.clear_color(Color::new(0.1, 0.2, 0.3, 1.0), &view);
    
        let camera_bind_group = self.get_camera_bind_group(camera);
        let (model_buffer, model_bind_group) = self.get_model_bind_group();

        let max_num_vertices = self.models.iter().map(|m| m.mesh.vertices.len()).max().unwrap_or(0);
        let vertex_buffer = Mesh::get_vertex_buffer(self.device, max_num_vertices);

        let max_indices = self.models.iter().map(|m| m.mesh.triangles.len()).max().unwrap_or(0) * 3;
        let index_buffer = Mesh::get_index_buffer(self.device, max_indices);

        for model in &self.models
        {
            let index_count = (model.mesh.triangles.len() * 3) as u64;
            let vertex_count = model.mesh.vertices.len() as u64;

            self.queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(model.mesh.vertices.as_slice()));
            self.queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(model.mesh.get_triangles().as_slice()));
            let model_uniform = model.get_model_uniform();
            self.queue.write_buffer(&model_buffer, 0, bytemuck::cast_slice(&[model_uniform]));

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
            {
                label: Some("Render Encoder")
            });

            let mut render_pass = self.get_render_pass(&mut encoder, &view);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &model_bind_group, &[]);

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(0..vertex_count));
            render_pass.set_index_buffer(index_buffer.slice(0..((index_count as usize * std::mem::size_of::<u16>()) as u64)), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(index_count as u32), 0, 0..1);
            drop(render_pass);

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        
        output.present();
        self.models.clear();
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

    fn get_model_bind_group(&self) -> (wgpu::Buffer, wgpu::BindGroup)
    {
        let model_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: &vec![0 as u8; std::mem::size_of::<ModelUniform>()],
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let model_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.model_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        (model_buffer, model_bind_group)
    }
}
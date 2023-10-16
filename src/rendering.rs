pub mod renderer;
pub mod debug_rendering;
pub mod mesh;

use std::sync::{Arc, Mutex};

use crate::{math::*, voxel::{VoxelStorage, Voxel, terrain_renderer::TerrainRenderStage, terrain::VoxelTerrain}, camera::Camera};
use crate::gpu_utils::*;

use wgpu::{VertexBufferLayout, BindGroupLayout};

use self::{renderer::Renderer, debug_rendering::{DebugRenderStage, DebugObject}, mesh::{MeshRenderStage, Mesh, MeshInstance}};

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

const TEST_SHADER: &[u8] = include_bytes!(env!("test_shader.spv"));

struct TestRenderStage
{
    pipeline: wgpu::RenderPipeline
}

impl TestRenderStage
{
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        println!("Shader byte length: {}", TEST_SHADER.len());

        let shader = &device.create_shader_module(wgpu::include_spirv!(env!("test_shader.spv")));
        
        let pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo 
        { 
            shader, 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[], 
            bind_groups: &[], 
            label: Some("Test Render Pipeline") 
        });
        
        Self 
        {
            pipeline
        }
    }
}

impl RenderStage for TestRenderStage
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.pipeline
    }
    
    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        vec![Box::new(TestDrawCall())]
    }
}

struct TestDrawCall();

impl DrawCall for TestDrawCall
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([])
    }
    
    fn on_pre_draw(&self, _queue: &wgpu::Queue) {}
    
    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.draw(0..3, 0..1);
    }
}
pub struct GameRenderer<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    renderer: Renderer,
    debug_stage: DebugRenderStage,
    mesh_stage: MeshRenderStage,
    terrain_stage: TerrainRenderStage<TStorage>,

    test_stage: TestRenderStage
}

impl<TStorage> GameRenderer<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    pub fn new(terrain: Arc<Mutex<VoxelTerrain<TStorage>>>, camera: Camera, device: Arc<wgpu::Device>, surface: Arc<wgpu::Surface>, queue: Arc<wgpu::Queue>, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let clear_color = Color::new(0.1, 0.2, 0.3, 1.0);
        let renderer = Renderer::new(device.clone(), surface, queue, config, clear_color);

        let debug_stage = DebugRenderStage::new(device.clone(), config, camera.clone(), &[]);
        let mesh_stage = MeshRenderStage::new(Mesh::cube(Color::RED), &[MeshInstance::from_position([0.0, 2.0, 0.0].into())], camera.clone(), &device, config);

        let terrain_stage = TerrainRenderStage::new(terrain, camera.clone(), device.clone(), config);

        Self 
        { 
            renderer, 
            debug_stage, 
            mesh_stage, 
            terrain_stage,
            test_stage: TestRenderStage::new(&device, config)
        }
    }

    pub fn update(&mut self, camera: &Camera, debug_objects: &[DebugObject])
    {
        self.debug_stage.update(debug_objects, camera.clone());
        self.mesh_stage.update(camera.clone());
        self.terrain_stage.update(camera.clone());
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError>
    {
        self.renderer.render(&[&self.test_stage])
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration)
    {
        self.renderer.resize(config);
    }
}
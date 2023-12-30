
use std::sync::Arc;

use vox_engine::app::WinitWindow;
use vox_engine::app::input::FrameState;
use vox_engine::gpu_utils::WgpuState;
use vox_engine::math::Color;
use vox_engine::prelude::vox_core::terrain::TerrainArgs;
use vox_engine::prelude::vox_core::{VoxelModel, VoxelModelInstance};
use vox_engine::rendering::Renderer;
use vox_engine::rendering::camera::Camera;
use vox_engine::utils::{Wrapper, Array3D};
use vox_engine::voxel::terrain::TerrainGenerator;
use vox_engine::voxel::{build_voxel_models, MODEL_3X3X3, MODEL_TEAPOT, MODEL_MONUMENT, SANDSTONE, TREE_BARK, GRANITE, TREE_LEAVES, WATER, ERROR};
use vox_engine::voxel::voxel_renderer::VoxelRenderer;
use vox_engine::wgpu::SurfaceError;
use vox_engine::winit::event_loop::EventLoop;
use vox_engine::winit::event::Event;
use vox_engine::rendering::gui::{GuiRenderer, GuiRendererDescriptor};
use vox_engine::glam::{UVec2, Vec3, IVec3};
use vox_engine::wgpu;

use crate::GUI_SAVE_PATH;
use crate::world::GameWorld;

#[derive(Clone, Copy)]
pub struct RenderData <'a>
{
    pub frame_state: &'a FrameState,
    pub gpu_state: &'a WgpuState,
}

pub struct GameRenderer
{
    renderer: Renderer,
    gui_renderer: GuiRenderer,
    voxel_renderer: VoxelRenderer,
}

impl GameRenderer
{
    pub fn new<T>(gpu_state: &WgpuState, event_loop: &EventLoop<T>, window: Arc<WinitWindow>, camera: &Camera) -> Self
        where T : Sized
    {
        let renderer = Renderer::new(gpu_state.device().clone(), gpu_state.surface().clone(), gpu_state.queue().clone(), &gpu_state.surface_config(), Color::new(0.2, 0.2, 0.2, 1.0));
        
        let mut gui_renderer = GuiRenderer::new(GuiRendererDescriptor { 
            event_loop, 
            device: &gpu_state.device(), 
            rt_format: gpu_state.surface_config().format,
            window
        });

        gui_renderer.load(GUI_SAVE_PATH);

        const CHUNK_SIZE: u32 = 100;
        const SEED: u32 = 1;

        let voxel_array = generate_terrain(gpu_state.device().clone(), gpu_state.queue().clone(), CHUNK_SIZE, SEED);
        let model = VoxelModel::new(CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE, 0);
        let model_instance = VoxelModelInstance::new(Vec3::ZERO, 1.0, model);
        let voxel_renderer = VoxelRenderer::new(&gpu_state, camera, voxel_array.as_slice(), &[Wrapper(model_instance)]);

        Self 
        { 
            renderer, 
            gui_renderer, 
            voxel_renderer,
        }
    }

    pub fn on_exit(&self)
    {
        self.gui_renderer.save(GUI_SAVE_PATH);
    }

    pub fn on_resize(&mut self, new_size: UVec2, gpu_state: &WgpuState)
    {
        if new_size.x > 0 && new_size.y > 0
        {
            self.renderer.resize(&gpu_state.surface_config());
            self.voxel_renderer.resize(gpu_state.queue(), gpu_state.device(), gpu_state.surface_config());
        }
    }

    pub fn gui_handle_event<'a, T>(&mut self, event: &Event<'a, T>) -> bool 
    {
        self.gui_renderer.handle_event(event)
    }

    pub fn render_world(&mut self, world: &mut GameWorld, render_data: RenderData) -> Result<(), SurfaceError>
    {
        self.gui_renderer.begin_frame();
        self.gui_renderer.draw_ui(|c| {
            world.on_gui(render_data.frame_state, c)
        });
        self.gui_renderer.end_frame();

        self.voxel_renderer.update(world.main_camera.camera(), &render_data.gpu_state.queue());
        
        let _ = self.voxel_renderer.get_profiling_info();
        
        self.renderer.render(&mut [&mut self.voxel_renderer, &mut self.gui_renderer])

    }
}

fn generate_terrain(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, chunk_size: u32, seed: u32) -> Array3D<u32>
{
    let args = TerrainArgs {
        chunk_size,
        seed,
    };

    let mut terrain_generator = TerrainGenerator::new(device, queue, args);
    let data = terrain_generator.generate(IVec3::new(2, 0, 0));
    data
}

fn load_voxels() -> (Vec<u32>, Vec<VoxelModel>)
{
    let (models, voxels) = build_voxel_models(&[MODEL_3X3X3, MODEL_TEAPOT, MODEL_MONUMENT], |i| {
        match i
        {
            121 => SANDSTONE.id,
            122 => TREE_BARK.id,
            123 => GRANITE.id,
            81 => TREE_LEAVES.id,
            97 => WATER.id,
            _ => ERROR.id
        }
    }).unwrap();

    (voxels, models)
}
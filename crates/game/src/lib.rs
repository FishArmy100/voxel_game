pub mod world;
pub mod renderer;

use std::sync::Arc;
use std::time::SystemTime;

use renderer::{GameRenderer, RenderData};
use vox_engine::app::input::{FrameStateBuilder, FrameState, KeyCode};
use vox_engine::app::{App, WinitWindow, self};
use vox_engine::math::{Color, Vec3};
use vox_engine::rendering::Renderer;
use vox_engine::rendering::camera::{CameraEntity, Camera};
use vox_engine::rendering::gui::GuiRendererDescriptor;
use vox_engine::wgpu::SurfaceError;
use vox_engine::winit::event::{Event, WindowEvent, ElementState, KeyboardInput};
use vox_engine::winit::event_loop::{EventLoop, ControlFlow};
use vox_engine::winit::window::Window;
use vox_engine::gpu_utils::WgpuState;
use world::GameWorld;
use vox_engine::glam::UVec2;

pub const GUI_SAVE_PATH: &str = "gui_data.json";
pub const WINDOW_TITLE: &str = "Voxel Game";

pub struct GameApp
{
    window: Arc<WinitWindow>,
    gpu_state: WgpuState,

    world: GameWorld,
    renderer: GameRenderer,

    frame_builder: FrameStateBuilder,
    current_frame: FrameState,

    current_time: SystemTime,
    window_size: UVec2,
}

impl GameApp
{
    fn resize(&mut self, new_size: UVec2)
    {
        if new_size.x > 0 && new_size.y > 0
        {
            self.gpu_state.resize(UVec2::new(new_size.x, new_size.y));
            self.renderer.on_resize(new_size, &self.gpu_state);
        }
    }

    fn on_render(&mut self) -> Result<(), vox_engine::wgpu::SurfaceError>
    {
        self.renderer.render_world(&mut self.world, RenderData {
            frame_state: &self.current_frame,
            gpu_state: &self.gpu_state
        })
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        let frame_state = self.frame_builder.build(delta_time);

        self.world.on_update(&frame_state);
        self.current_time = SystemTime::now();

        self.current_frame = frame_state.clone();
        self.frame_builder = FrameStateBuilder::new(self.window.clone(), frame_state);
    }
}

impl App for GameApp
{
    fn new<T>(event_loop: &EventLoop<T>, window: Window) -> Self
        where T : Sized 
    {
        window.set_title(WINDOW_TITLE);
        let window = Arc::new(window);
        let gpu_state = pollster::block_on(WgpuState::new(&window));

        let world = GameWorld::new();
        let renderer = GameRenderer::new(&gpu_state, event_loop, window.clone(), world.main_camera.camera());
        
        let start_frame = FrameState::new(&window);
        let frame_builder = FrameStateBuilder::new(window.clone(), start_frame.clone());

        let window_size = [window.inner_size().width, window.inner_size().height].into();

        Self 
        { 
            window, 
            gpu_state,
            world,
            renderer,
            frame_builder,
            current_frame: start_frame,
            current_time: SystemTime::now(),
            window_size,
        }
    }

    fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow) 
    {
        if self.renderer.gui_handle_event(&event)
        {
            return;
        }

        self.frame_builder.on_event(&event);
        match event 
        {
            Event::WindowEvent {
                ref event,
                window_id,
            } 

            if window_id == self.window.id() =>
            {
                match event 
                {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        self.resize([physical_size.width, physical_size.height].into());
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.resize([new_inner_size.width, new_inner_size.height].into());
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.on_update();
                match self.on_render()
                {
                    Ok(_) => {},
                    Err(SurfaceError::Lost) => self.resize(self.window_size),
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            }

            Event::MainEventsCleared => {
                self.window.request_redraw();
            },

            Event::LoopDestroyed => {
                self.renderer.on_exit()
            }
            _ => {}
        }
    }
}

pub fn run()
{
    println!("Running: {}", WINDOW_TITLE);
    app::run::<GameApp>()
}

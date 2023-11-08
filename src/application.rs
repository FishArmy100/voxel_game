pub mod input;

use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::{time::SystemTime, sync::Arc};
use winit::event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState, MouseButton, MouseScrollDelta, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::gpu_utils::WgpuState;
use crate::rendering::GameRenderer;

use crate::math::{Vec3, Color, Vec2};
use crate::camera::{Camera, CameraEntity};

pub type WinitWindow = winit::window::Window;
pub type WindowSize = winit::dpi::PhysicalSize<u32>;
pub type WindowPosition = winit::dpi::PhysicalPosition<u32>;
use self::input::*;

struct AppState
{
    app_name: String,
    current_time: SystemTime,
    frame_builder: FrameStateBuilder,

    size: WindowSize,
    window_handle: Arc<WinitWindow>,
    renderer: GameRenderer,

    wgpu_state: WgpuState,

    // TEMP
    camera_entity: CameraEntity,
}

pub async fn run()
{
    let name = "Voxel Game";
    let (event_loop, window) = get_window();
    let mut app_state = AppState::new(name, &event_loop, window).await;

    event_loop.run(move |event, _, control_flow| {
        app_state.on_event(event, control_flow)
    })
}

fn get_window() -> (EventLoop<()>, WinitWindow)
{
    let event_loop = EventLoop::new();
    let window = WinitWindow::new(&event_loop).unwrap();
    (event_loop, window)  
}

impl AppState
{
    async fn new<T>(name: &str, event_loop: &EventLoop<T>, window: WinitWindow) -> Self
        where T : 'static
    {
        window.set_title(name);
        let wgpu_state = WgpuState::new(&window).await;
        let window_handle = Arc::new(window);
        let size = window_handle.inner_size();

        let aspect = wgpu_state.surface_config().width as f32 / wgpu_state.surface_config().height as f32;

        
        let camera = Camera
        {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::unit_y(),
            aspect,
            fov: 45.0,
            near: 0.1,
            far: 100000.0
        };

        let renderer = GameRenderer::new(camera.clone(), wgpu_state.device().clone(), wgpu_state.surface().clone(), wgpu_state.queue().clone(), &wgpu_state.surface_config(), event_loop, window_handle.clone());
        let frame_builder = FrameStateBuilder::new(window_handle.clone(), FrameState::new(&window_handle));

        Self
        {
            app_name: name.into(),
            current_time: SystemTime::now(),
            frame_builder,
            size,
            window_handle,
            wgpu_state,
            renderer,
            camera_entity: CameraEntity::new(camera, 20.0, 50.0, 80.0),
        }
    }

    fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow)
    {
        if self.renderer.handle_event(&event)
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

            if window_id == self.window_handle.id() =>
            {
                match event 
                {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == self.window_handle.id() => {
                self.on_update();
                match self.on_render()
                {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            }

            Event::MainEventsCleared => {
                self.window_handle.request_redraw();
            },

            Event::LoopDestroyed => {
                self.renderer.on_close();
            }
            _ => {}
        }
    }

    fn resize(&mut self, new_size: WindowSize)
    {
        if new_size.width > 0 && new_size.height > 0
        {
            self.size = new_size;
            self.wgpu_state.resize(Vec2::new(new_size.width, new_size.height));
            self.renderer.resize(&self.wgpu_state.surface_config());

            self.camera_entity.mut_camera().aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn on_render(&mut self) -> Result<(), wgpu::SurfaceError>
    {        
        self.renderer.render()?;
        Ok(())
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        let frame_state = self.frame_builder.build(delta_time);

        self.camera_entity.update(&frame_state);
        self.renderer.update(self.camera_entity.camera(), &vec![], delta_time);
        self.current_time = SystemTime::now();

        self.frame_builder = FrameStateBuilder::new(self.window_handle.clone(), frame_state);
    }
}


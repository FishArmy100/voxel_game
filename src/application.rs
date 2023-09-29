pub mod input;

use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::{time::SystemTime, sync::Arc};
use winit::event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState, MouseButton, MouseScrollDelta, DeviceEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::gpu::ShaderInfo;
use crate::rendering::GameRenderer;
use crate::voxel::brick_map::{BrickMap, SizedBrickMap};
use crate::voxel::octree::Octree;
use crate::voxel::{Voxel, VoxelData, VoxelStorage};

use crate::colors::Color;
use crate::math::{Vec3, Point3D, Vec2};
use crate::camera::{Camera, CameraEntity};
use crate::voxel::terrain::{VoxelTerrain, TerrainInfo};

pub type WinitWindow = winit::window::Window;
pub type WindowSize = winit::dpi::PhysicalSize<u32>;
pub type WindowPosition = winit::dpi::PhysicalPosition<u32>;
use self::input::*;

type Storage = SizedBrickMap<Voxel, 4>;

struct AppState
{
    app_name: String,
    current_time: SystemTime,
    frame_builder: FrameStateBuilder,

    surface: Arc<wgpu::Surface>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    size: WindowSize,
    window_handle: Arc<WinitWindow>,
    renderer: GameRenderer<Storage>,

    // TEMP
    camera_entity: CameraEntity,
    terrain: Arc<Mutex<VoxelTerrain<Storage>>>,
}

pub async fn run()
{
    let name = "Voxel Game";
    let (event_loop, window) = get_window();
    let mut app_state = AppState::new(name, window).await;

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
    async fn new(name: &str, window: WinitWindow) -> Self
    {
        window.set_title(name);

        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default()
        });

        let surface = unsafe {instance.create_surface(&window)}.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions 
            { 
                power_preference: wgpu::PowerPreference::default(), 
                compatible_surface: Some(&surface), 
                force_fallback_adapter: false
            }
        ).await.unwrap();

        println!("Name: {:?}\nBackend: {:?}", adapter.get_info().name, adapter.get_info().backend);

        let (device, queue) = adapter.request_device( 
            &wgpu::DeviceDescriptor
            {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None
            }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };

        surface.configure(&device, &config);

        let surface = Arc::new(surface);
        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let window_handle = Arc::new(window);

        let camera = Camera
        {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100000.0
        };

        let terrain = generate_terrain(device.clone(), queue.clone());

        let renderer = GameRenderer::new(terrain.clone(), camera.clone(), device.clone(), surface.clone(), queue.clone(), &config);
        let frame_builder = FrameStateBuilder::new(window_handle.clone(), FrameState::new(&window_handle));

        Self
        {
            app_name: name.into(),
            current_time: SystemTime::now(),
            frame_builder,
            surface,
            device,
            queue,
            config,
            size,
            window_handle,
            renderer,
            camera_entity: CameraEntity::new(camera, 20.0, 50.0, 80.0),
            terrain,
        }
    }

    fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow)
    {
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
            _ => {}
        }
    }

    fn resize(&mut self, new_size: WindowSize)
    {
        if new_size.width > 0 && new_size.height > 0
        {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.device.poll(wgpu::MaintainBase::Wait); // to fix crash on dx12 with wgpu 0.17
            self.surface.configure(&self.device, &self.config);
            self.renderer.resize(&self.config);

            self.camera_entity.mut_camera().aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn on_render(&mut self) -> Result<(), wgpu::SurfaceError>
    {        
        let debug_objs = vec![];
        self.renderer.update(self.camera_entity.camera(), &debug_objs);

        self.renderer.render()?;
        Ok(())
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        let frame_state = self.frame_builder.build(delta_time);

        self.camera_entity.update(&frame_state);
        // println!("{}ms", delta_time * 1000.0);
        self.current_time = SystemTime::now();
        self.terrain.lock().unwrap().tick();

        self.frame_builder = FrameStateBuilder::new(self.window_handle.clone(), frame_state);
    }
}

fn generate_terrain<TStorage>(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Arc<Mutex<VoxelTerrain<TStorage>>> 
    where TStorage : VoxelStorage<Voxel> + Send + 'static
{        
    let sand_color = Color::new(0.76, 0.698, 0.502, 1.0);

    let voxel_types = vec!
    [
        VoxelData::new(Color::WHITE), 
        VoxelData::new(Color::BLUE),
        VoxelData::new(sand_color),
        VoxelData::new(Color::GREEN)
    ];
        
    const CHUNK_DEPTH: usize = 7;
    const VOXEL_SIZE: f32 = 1.0;

    let info = TerrainInfo
    {
        chunk_depth: CHUNK_DEPTH,
        voxel_size: VOXEL_SIZE,
        voxel_types: Arc::new(voxel_types),
    };

    let shader_info = ShaderInfo {
        entry_point: "main",
        source: include_str!("shaders/test_compute.wgsl")
    };

    let terrain = Arc::new(Mutex::new(VoxelTerrain::new(info, shader_info, device.clone(), queue))); 
    terrain.lock().unwrap().generate_chunk([0, 0, 0].into());

    terrain
}


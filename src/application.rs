use std::{time::SystemTime, sync::Arc};
use cgmath::Array;
use noise::{Perlin, NoiseFn, Seedable};
use winit::{event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState}, event_loop::{ControlFlow, EventLoop}};
use crate::{rendering::{Renderer, VoxelFaceData, VoxelFaces}, math::Point3D, voxel::{Voxel, VoxelData}, debug_utils};
use crate::colors::Color;
use crate::math::Vec3;
use crate::camera::{Camera, CameraEntity};
use crate::voxel::VoxelTerrain;

pub type WinitWindow = winit::window::Window;
pub type WindowSize = winit::dpi::PhysicalSize<u32>;

struct AppState
{
    app_name: String,
    current_time: SystemTime,

    surface: Arc<wgpu::Surface>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    size: WindowSize,
    window_handle: WinitWindow,

    // TEMP
    camera_entity: CameraEntity,
    terrain: VoxelTerrain
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

        
        let terrain_size_in_chunks = Vec3::new(10, 5, 10);

        let perlin = Perlin::new(326236);

        let generator = |pos: Vec3<usize>| 
        {
            let noise_value = (perlin.get([pos.x as f64 / 30.494948, pos.z as f64 / 30.494948]) * (16 * terrain_size_in_chunks.y) as f64) as f32 / 3.;

            if noise_value > pos.y as f32
            {
                if (pos.x % 2 == 1) ^ (pos.z % 2 == 1)
                {
                    Voxel::new(1)
                }
                else 
                {
                    Voxel::new(2)
                }
            } 
            else 
            {
                Voxel::new(0)
            }
        };
        
        let voxel_types = Arc::new(vec!
        [
            VoxelData::new(Color::BLACK, false), 
            VoxelData::new(Color::GREEN, true),
            VoxelData::new(Color::RED, true),
        ]);

        let terrain = VoxelTerrain::new(Point3D::from_value(0.0), terrain_size_in_chunks, 1., voxel_types, &generator);

        Self
        {
            app_name: String::from(name),
            current_time: SystemTime::now(),
            surface: Arc::new(surface),
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
            size,
            window_handle: window,
            camera_entity: CameraEntity::new(camera, 20., 50.),
            terrain
        }
    }

    fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow)
    {
        match event 
        {
            Event::WindowEvent {
                ref event,
                window_id,
            } 

            if window_id == self.window_handle.id() => if !self.camera_entity.on_event(event)
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
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn on_render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let renderer = &mut Renderer::new(self.device.clone(), self.surface.clone(), self.queue.clone(), &self.config);
        self.terrain.render(renderer, self.camera_entity.camera())
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        self.camera_entity.update(delta_time); 
        println!("{}ms", delta_time * 1000.0);
        self.current_time = SystemTime::now();
    }
}


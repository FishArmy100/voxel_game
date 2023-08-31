use std::{time::SystemTime, sync::Arc};
use cgmath::{Zero, Array};
use noise::{Perlin, NoiseFn};
use winit::event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::rendering::GameRenderer;
use crate::rendering::debug_render_stage::{DebugLine, self, DebugRenderStage, DebugObject, DebugCube};
use crate::rendering::renderer::Renderer;
use crate::rendering::voxel_render_stage::VoxelRenderStage;
use crate::voxel::octree::{Octree, VisitedNodeType};
use crate::voxel::{Voxel, VoxelData};

use crate::colors::Color;
use crate::math::{Vec3, Point3D};
use crate::camera::{Camera, CameraEntity};
use crate::voxel::terrain::VoxelTerrain;

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
    renderer: GameRenderer,

    // TEMP
    camera_entity: CameraEntity,
    terrain: Arc<VoxelTerrain>,
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

        let terrain_size_in_chunks = Vec3::new(8, 5, 8);

        let perlin = Perlin::new(326236);

        let generator = |pos: Vec3<usize>| 
        {
            let noise_value = (perlin.get([pos.x as f64 / 30.494948, pos.z as f64 / 30.494948]) * (16 * terrain_size_in_chunks.y) as f64) as f32 / 4.;
            let water_height = terrain_size_in_chunks.y as f32 * 16. / 10.0;
            let sand_height = water_height + 2.0;

            if pos.y as f32 <= water_height
            {
                Some(Voxel::new(1))
            }
            else if pos.y as f32 <= noise_value
            {
                if pos.y as f32 <= sand_height
                {
                    Some(Voxel::new(2))
                }
                else 
                {
                    Some(Voxel::new(3))
                }
            } 
            else 
            {
                None
            }
        };

        let generator2 = |position: Vec3<usize>| {
            if position.y == 0
            {
                if (position.x % 2 == 0) ^ (position.z % 2 == 0)
                {
                    Some(Voxel::new(1))
                }
                else 
                {
                    Some(Voxel::new(3))
                }
            }
            else 
            {
                None
            }
        };
        
        let sand_color = Color::new(0.76, 0.698, 0.502, 1.0);

        let voxel_types = vec!
        [
            VoxelData::new(Color::WHITE), 
            VoxelData::new(Color::BLUE),
            VoxelData::new(sand_color),
            VoxelData::new(Color::GREEN)
        ];
        
        const CHUNK_DEPTH: usize = 4;
        const VOXEL_SIZE: f32 = 1.0;

        let terrain_pos = Point3D::new(0.0, 0.0, 0.0);
        let terrain = Arc::new(VoxelTerrain::new(terrain_pos, terrain_size_in_chunks, CHUNK_DEPTH, VOXEL_SIZE, voxel_types, device.clone(), &generator));

        let renderer = GameRenderer::new(terrain.clone(), camera.clone(), device.clone(), surface.clone(), queue.clone(), &config);

        let mut octree = Octree::new(4);
        octree.insert(Vec3::new(0, 4, 2), Some(0));
        
        println!("Value: {:?}", octree.get(Vec3::new(0, 7, 2)));

        Self
        {
            app_name: name.into(),
            current_time: SystemTime::now(),
            surface,
            device,
            queue,
            config,
            size,
            window_handle: window,
            renderer,
            camera_entity: CameraEntity::new(camera, 20., 50.),
            terrain,
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
            self.renderer.resize(&self.config);

            self.camera_entity.mut_camera().aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn on_render(&mut self) -> Result<(), wgpu::SurfaceError>
    {        
        let mut debug_objs = vec![];
        let debug_line = DebugObject::Line(DebugLine::new(Vec3::new(0.0, 20.0, 0.0), Vec3::new(128.0, 20.0, 0.0), Color::BLACK));
        debug_objs.push(debug_line);
        self.renderer.update(self.camera_entity.camera(), &debug_objs);

        self.renderer.render()?;
        Ok(())
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        self.camera_entity.update(delta_time); 
        println!("{}ms", delta_time * 1000.0);
        self.current_time = SystemTime::now();
    }
}


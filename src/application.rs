use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::{time::SystemTime, sync::Arc};
use cgmath::{Zero, Array};
use noise::{Perlin, NoiseFn};
use winit::event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::rendering::GameRenderer;
use crate::rendering::debug_render_stage::{DebugLine, self, DebugRenderStage, DebugObject, DebugCube};
use crate::rendering::renderer::Renderer;
use crate::rendering::voxel_render_stage::VoxelRenderStage;
use crate::voxel::octree::{Octree, VisitedNodeType};
use crate::voxel::{Voxel, VoxelData};

use crate::colors::Color;
use crate::math::{Vec3, Point3D, Vec2};
use crate::camera::{Camera, CameraEntity};
use crate::voxel::terrain::{VoxelTerrain, TerrainInfo, VoxelGenerator};

pub type WinitWindow = winit::window::Window;
pub type WindowSize = winit::dpi::PhysicalSize<u32>;
pub type WindowPosition = winit::dpi::PhysicalPosition<u32>;

struct DefaultVoxelGenerator
{
    perlin: Perlin
}

impl VoxelGenerator for DefaultVoxelGenerator
{
    fn get(&self, index: Vec3<isize>) -> Option<Voxel> 
    {
        let noise_value = (self.perlin.get([index.x as f64 / 30.494948, index.z as f64 / 30.494948]) * (16 * 2) as f64) as f32 / 4.;
        let water_height = 2 as f32 * 16. / 10.0;
        let sand_height = water_height + 2.0; 

        if index.y as f32 <= water_height
        {
            Some(Voxel::new(1))
        }
        else if index.y as f32 <= noise_value
        {
            if index.y as f32 <= sand_height
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
    }
}

#[derive(Debug)]
pub struct FrameState
{
    keys_pressed: Vec<VirtualKeyCode>,
    keys_released: Vec<VirtualKeyCode>,
    keys_down: Vec<VirtualKeyCode>,

    mouse_delta: Vec2<f64>,
    mouse_buttons_pressed: Vec<MouseButton>,
    mouse_buttons_released: Vec<MouseButton>,
    mouse_buttons_down: Vec<MouseButton>,
    mouse_scroll_delta: Option<MouseScrollDelta>,

    window_size: WindowSize,
    delta_time: f32
}

impl FrameState
{
    pub fn is_key_down(&self, keycode: VirtualKeyCode) -> bool { self.keys_down.contains(&keycode) }
    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool { self.keys_pressed.contains(&keycode) }
    pub fn is_key_released(&self, keycode: VirtualKeyCode) -> bool { self.keys_released.contains(&keycode) }

    pub fn is_mouse_button_down(&self, mouse_button: MouseButton) -> bool { self.mouse_buttons_down.contains(&mouse_button) }
    pub fn is_mouse_button_pressed(&self, mouse_button: MouseButton) -> bool { self.mouse_buttons_pressed.contains(&mouse_button) }
    pub fn is_mouse_button_released(&self, mouse_button: MouseButton) -> bool { self.mouse_buttons_released.contains(&mouse_button) }

    pub fn delta_time(&self) -> f32 { self.delta_time }

    fn new(window: &WinitWindow) -> Self
    {
        Self 
        {
            keys_pressed: vec![], 
            keys_released: vec![], 
            keys_down: vec![], 
            mouse_delta: Vec2::new(0.0, 0.0),
            mouse_buttons_pressed: vec![], 
            mouse_buttons_released: vec![], 
            mouse_buttons_down: vec![], 
            mouse_scroll_delta: None, 
            window_size: window.inner_size(),
            delta_time: 0.0
        }
    }
}

pub struct FrameStateBuilder
{
    window: Arc<WinitWindow>,

    keys_pressed: Vec<VirtualKeyCode>,
    keys_released: Vec<VirtualKeyCode>,
    keys_down: Vec<VirtualKeyCode>,

    mouse_delta: Vec2<f64>,
    mouse_buttons_pressed: Vec<MouseButton>,
    mouse_buttons_released: Vec<MouseButton>,
    mouse_buttons_down: Vec<MouseButton>,
    mouse_scroll_delta: Option<MouseScrollDelta>,

    window_size: WindowSize,
}

impl FrameStateBuilder
{
    pub fn new(window: Arc<WinitWindow>, previous_frame: FrameState) -> Self
    {
        let keys_down = previous_frame.keys_down.clone();
        let mouse_buttons_down = previous_frame.mouse_buttons_down.clone();
        let window_size = window.inner_size();

        Self 
        {
            window,
            keys_pressed: vec![], 
            keys_released: vec![], 
            keys_down, 
            mouse_delta: Vec2::new(0.0, 0.0),
            mouse_buttons_pressed: vec![], 
            mouse_buttons_released: vec![], 
            mouse_buttons_down, 
            mouse_scroll_delta: None, 
            window_size
        }
    }

    pub fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow)
    {
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
                    WindowEvent::KeyboardInput 
                    { 
                        input: KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                        ..
                    } => 
                    {
                        match state
                        {
                            ElementState::Pressed => 
                            {
                                self.keys_pressed.push(*keycode);
                                self.keys_down.push(*keycode);
                            },
                            ElementState::Released => 
                            {
                                self.keys_down.retain(|&x| x != *keycode);
                                self.keys_released.push(*keycode);
                            },
                        }
                    }
                    WindowEvent::MouseInput 
                    { 
                        state, 
                        button,
                        ..
                    } => 
                    {
                        match state
                        {
                            ElementState::Pressed => 
                            {
                                self.mouse_buttons_pressed.push(*button);
                                self.mouse_buttons_down.push(*button);
                            },
                            ElementState::Released => 
                            {
                                self.mouse_buttons_down.retain(|&b| b != *button);
                                self.mouse_buttons_released.push(*button);
                            },
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn build(self, delta_time: f32) -> FrameState
    {
        FrameState 
        { 
            keys_pressed: self.keys_pressed, 
            keys_released: self.keys_released, 
            keys_down: self.keys_down, 
            mouse_delta: self.mouse_delta, 
            mouse_buttons_pressed: self.mouse_buttons_pressed, 
            mouse_buttons_released: self.mouse_buttons_released, 
            mouse_buttons_down: self.mouse_buttons_down, 
            mouse_scroll_delta: self.mouse_scroll_delta, 
            window_size: self.window_size,
            delta_time
        }
    }
}

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
    renderer: GameRenderer,

    // TEMP
    camera_entity: CameraEntity,
    terrain: Arc<Mutex<VoxelTerrain>>,
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

        let perlin = Perlin::new(326236);

        let generator = Arc::new(DefaultVoxelGenerator {
            perlin
        });
        
        let sand_color = Color::new(0.76, 0.698, 0.502, 1.0);

        let voxel_types = vec!
        [
            VoxelData::new(Color::WHITE), 
            VoxelData::new(Color::BLUE),
            VoxelData::new(sand_color),
            VoxelData::new(Color::GREEN)
        ];
        
        const CHUNK_DEPTH: usize = 6;
        const VOXEL_SIZE: f32 = 1.0;

        let info = TerrainInfo
        {
            chunk_depth: CHUNK_DEPTH,
            voxel_size: VOXEL_SIZE,
            voxel_types: Arc::new(voxel_types),
        };

        let terrain = Arc::new(Mutex::new(VoxelTerrain::new(info, device.clone(), generator)));

        let mut current = 0;
        for x in 0..4
        {
            for y in 0..4
            {
                for z in 0..4
                {
                    let was_generated = terrain.lock().unwrap().generate_chunk([x, y, z].into());
                    current += 1;
                    println!("enqueued chunk {}/{}: {}", current, 4 * 4 * 4, was_generated);
                }
            }
        }
        

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
        let debug_objs = vec![];
        self.renderer.update(self.camera_entity.camera(), &debug_objs);

        self.renderer.render()?;
        Ok(())
    }

    fn on_update(&mut self)
    {
        let delta_time = self.current_time.elapsed().unwrap().as_secs_f32();
        self.camera_entity.update(delta_time); 
        //println!("{}ms", delta_time * 1000.0);
        self.current_time = SystemTime::now();
        self.terrain.lock().unwrap().tick();
    }
}


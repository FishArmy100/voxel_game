use std::{time::SystemTime, str::FromStr};
use crate::math::{Vec2};

type WinitWindow = winit::window::Window;
type WindowSize = winit::dpi::PhysicalSize<u32>;

pub struct App 
{
    app_name: String,
    current_time: SystemTime,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window_handle: WinitWindow,

    event_loop: winit::event_loop::EventLoop<()>
}

impl App 
{
    pub fn window_size(&self) -> WindowSize { self.size }

    pub async fn new(name: &str) -> Self
    {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
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
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            app_name: String::from(name),
            current_time: SystemTime::now(),
            surface,
            device,
            queue,
            config,
            size: window_size,
            window_handle: window,
            event_loop
        }
    }

    pub fn run(&mut self)
    {
        
    }

    fn on_resize(new_size: WindowSize)
    {

    }
}
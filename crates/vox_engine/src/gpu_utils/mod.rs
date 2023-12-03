pub mod bind_group;
pub mod buffer;
pub mod texture;
use std::sync::Arc;

use crate::math::{Vec4, Vec3};
use glam::UVec2;
use crate::utils::Byteable;

pub use self::bind_group::*;
pub use self::buffer::*;
pub use self::texture::*;

pub struct WgpuState
{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: Arc<wgpu::Surface>,
    surface_config: wgpu::SurfaceConfiguration
}

impl WgpuState
{
    pub fn device(&self) -> &Arc<wgpu::Device> { &self.device }
    pub fn queue(&self) -> &Arc<wgpu::Queue> { &self.queue }
    pub fn surface(&self) -> &Arc<wgpu::Surface> { &self.surface }
    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration { &self.surface_config }

    pub async fn new(window: &winit::window::Window) -> Self 
    {
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

        let features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        let (device, queue) = adapter.request_device( 
            &wgpu::DeviceDescriptor
            {
                features,
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

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let surface = Arc::new(surface);

        Self
        {
            device,
            queue,
            surface,
            surface_config: config
        }
    }

    pub fn resize(&mut self, size: UVec2)
    {
        if size.x > 0 && size.y > 0
        {
            self.surface_config.width = size.x;
            self.surface_config.height = size.y;
            self.device.poll(wgpu::MaintainBase::Wait); // to fix crash on dx12 with wgpu 0.17
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}
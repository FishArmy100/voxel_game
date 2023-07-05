use std::{time::SystemTime, str::FromStr};
use crate::math::{Vec2};
use winit::{event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState}, event_loop::{ControlFlow, EventLoop}};
use crate::rendering::Renderer;

pub type WinitWindow = winit::window::Window;
pub type WindowSize = winit::dpi::PhysicalSize<u32>;

struct AppState
{
    app_name: String,
    current_time: SystemTime,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: WindowSize,
    window_handle: WinitWindow,
}

pub async fn run()
{

}

impl AppState
{
    async fn new(name: &str, window: WinitWindow) -> Self
    {
        
    }
}


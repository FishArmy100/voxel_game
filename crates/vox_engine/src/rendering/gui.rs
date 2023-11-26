use std::{sync::Arc, fs::File, io::{Write, Read}, error::Error};

use egui::FullOutput;
use winit::event_loop::EventLoop;
use egui_winit::egui::{Context, RawInput};
use crate::rendering::RenderStage;

use super::{get_command_encoder, get_render_pass};

pub const DEFAULT_SAVE_PATH: &str = "gui_data.yaml";

pub struct GuiRenderer
{
    context: egui::Context, 
    platform: egui_winit::State,
    renderer: egui_wgpu::renderer::Renderer,

    window: Arc<winit::window::Window>,
    full_output: FullOutput
}

impl GuiRenderer
{
    pub fn new<T>(desc: GuiRendererDescriptor<T>) -> Self
        where T : 'static
    {
        let GuiRendererDescriptor
        {
            event_loop,
            device,
            rt_format,
            window
        } = desc;

        let context = egui_winit::egui::Context::default();
        let platform = egui_winit::State::new(event_loop);
        let renderer = egui_wgpu::renderer::Renderer::new(device, rt_format, None, 1);

        Self 
        {
            context,
            platform,
            renderer,
            window,
            full_output: egui::FullOutput::default()
        }
    }

    pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> bool 
    {
        match event 
        {
            winit::event::Event::WindowEvent { event, .. } => 
            {
                self.platform.on_event(&self.context, event).consumed
            }
            _ => false
        }
    }

    pub fn save(&self, path: &str)
    {
        let yaml = self.context.memory(|m| {
            serde_json::to_string(m).expect("Could not serialize gui context memory")
        });

        let mut file = File::create(path)
            .expect(format!("Could not create file {}", path).as_str());

        file.write_all(yaml.as_bytes())
            .expect(format!("Could not write to file {}", path).as_str());
    }

    pub fn load(&mut self, path: &str)
    {
        if let Ok(mut file) = File::open(path)
        {
            let mut yaml = String::new();
            file.read_to_string(&mut yaml).expect(format!("Could not read file {}", path).as_str());

            let memory: egui::Memory = serde_json::from_str(&yaml)
                .expect("Could not deserialize gui context memory");

            self.context.memory_mut(|m| {
                *m = memory
            });
        }
    }

    pub fn begin_frame(&mut self)
    {
        let raw_input: RawInput = self.platform.take_egui_input(&self.window);
        self.context.begin_frame(raw_input);
    }

    pub fn end_frame(&mut self)
    {
        self.full_output = self.context.end_frame();
    }

    pub fn draw_ui<F>(&mut self, f: F) where F : FnOnce(&Context)
    {
        f(&self.context);
    }

}

impl RenderStage for GuiRenderer
{
    fn on_draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, _depth_texture: &crate::gpu_utils::Texture) 
    {
        let size = self.window.inner_size();
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor
        {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: self.window.scale_factor() as f32
        };

        let full_output = self.full_output.clone();
        self.platform.handle_platform_output(&self.window, &self.context, full_output.platform_output);
        let clipped_primitives = self.context.tessellate(full_output.shapes);
        let mut encoder = get_command_encoder(device);

        self.renderer.update_buffers(device, queue, &mut encoder, &clipped_primitives, &screen_descriptor);
        for (texture_id, image_delta) in full_output.textures_delta.set
        {
            self.renderer.update_texture(device, queue, texture_id, &image_delta);
        }

        for texture_id in full_output.textures_delta.free
        {
            self.renderer.free_texture(&texture_id);
        }

        let mut render_pass = get_render_pass(&mut encoder, view, None);
        self.renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        drop(render_pass);

        queue.submit(std::iter::once(encoder.finish()));
    }
}

pub struct GuiRendererDescriptor<'a, T> 
    where T : 'static
{
    pub event_loop: &'a EventLoop<T>,
    pub device: &'a wgpu::Device,
    pub rt_format: wgpu::TextureFormat,
    pub window: Arc<winit::window::Window>,
}
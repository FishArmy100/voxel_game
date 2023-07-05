
pub struct Renderer<'s, 'd, 'q>
{
    device: &'d wgpu::Device,
    surface: &'s wgpu::Surface,
    queue: &'q mut wgpu::Queue
}

impl<'s, 'd, 'q> Renderer<'s, 'd, 'q>
{
    pub fn new(device: &'d wgpu::Device, surface: &'s wgpu::Surface, queue: &'q mut wgpu::Queue) -> Self
    {
        Renderer { device, surface, queue }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("Render Encoder")
        });
        
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations{
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0
                    }),
                    store: true,
                }
            })],

            depth_stencil_attachment: None
        });
        
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
use winit::{event_loop::{EventLoop, ControlFlow}, window::Window, event::Event};

pub mod input;
pub type WinitWindow = Window;

pub fn run<T>() where T : App + 'static
{
    let (event_loop, window) = get_window();

    let mut app = T::new(&event_loop, window);

    event_loop.run(move |event, _, control_flow| {
        app.on_event(event, control_flow)
    });
}

pub trait App
{
    fn new<T>(event_loop: &EventLoop<T>, window: Window) -> Self
        where T : Sized;
    fn on_event<'a, T>(&mut self, event: Event<'a, T>, control_flow: &mut ControlFlow);
}

fn get_window() -> (EventLoop<()>, WinitWindow)
{
    let event_loop = EventLoop::new();
    let window = WinitWindow::new(&event_loop).unwrap();
    (event_loop, window)  
}
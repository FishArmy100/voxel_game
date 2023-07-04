mod test;
mod math;
mod colors;
mod texture;
mod camera;
mod application;
mod rendering;

use std::borrow::Cow;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder
};

use test::State;

async fn run() 
{
    let event_loop = EventLoop::new();
    
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| 
    {
        match event 
        {
            Event::WindowEvent {
                ref event,
                window_id,
            } 

            if window_id == state.window().id() => if !state.input(event) 
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render()
                {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e)
                }
            }

            Event::MainEventsCleared => {
                state.window().request_redraw();
            },
            _ => {}
        }
    });
}

fn main() 
{
    env_logger::init();
    pollster::block_on(run());
}
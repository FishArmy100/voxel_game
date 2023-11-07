use std::sync::Arc;

use cgmath::Zero;
use winit::event::{VirtualKeyCode, MouseButton, MouseScrollDelta, Event, KeyboardInput, ElementState, DeviceEvent};
use super::{WindowEvent, WindowSize, WinitWindow};

use crate::math::Vec2;



#[derive(Debug)]
pub struct FrameState
{
    keys_pressed: Vec<VirtualKeyCode>,
    keys_released: Vec<VirtualKeyCode>,
    keys_down: Vec<VirtualKeyCode>,

    mouse_delta: Vec2<f32>,
    mouse_position: Vec2<f32>,

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

    pub fn mouse_position(&self) -> Vec2<f32> { self.mouse_position }
    pub fn mouse_delta(&self) -> Vec2<f32> { self.mouse_delta }

    pub fn new(window: &WinitWindow) -> Self
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
            delta_time: 0.0,
            mouse_position: Vec2::new(0.0, 0.0)
        }
    }
}

pub struct FrameStateBuilder
{
    window: Arc<WinitWindow>,

    keys_pressed: Vec<VirtualKeyCode>,
    keys_released: Vec<VirtualKeyCode>,
    keys_down: Vec<VirtualKeyCode>,

    mouse_buttons_pressed: Vec<MouseButton>,
    mouse_buttons_released: Vec<MouseButton>,
    mouse_buttons_down: Vec<MouseButton>,
    mouse_scroll_delta: Option<MouseScrollDelta>,

    window_size: WindowSize,
    current_mouse_position: Vec2<f32>,
    mouse_delta: Vec2<f32>
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
            mouse_buttons_pressed: vec![], 
            mouse_buttons_released: vec![], 
            mouse_buttons_down, 
            mouse_scroll_delta: None, 
            window_size,
            current_mouse_position: previous_frame.mouse_position,
            mouse_delta: Vec2::zero()
        }
    }

    pub fn on_event<'a, T>(&mut self, event: &Event<'a, T>)
    {
        match event 
        {
            Event::WindowEvent {
                ref event,
                window_id,
            }

            if *window_id == self.window.id() =>
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

                    WindowEvent::MouseWheel 
                    { 
                        delta,
                        ..
                    } =>
                    {
                        self.mouse_scroll_delta = Some(*delta)
                    }

                    WindowEvent::CursorMoved 
                    {  
                        position, 
                        ..
                    } =>
                    {
                        self.current_mouse_position = Vec2::new(position.x as f32, position.y as f32)
                    }

                    _ => {}
                }
            },

            Event::DeviceEvent 
            { 
                ref event,
                device_id
            } => 
            {
                match event 
                {
                    DeviceEvent::MouseMotion 
                    { 
                        delta 
                    } =>
                    {
                        self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                    },

                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub fn build(&self, delta_time: f32) -> FrameState
    {
        FrameState 
        { 
            keys_pressed: self.keys_pressed.clone(), 
            keys_released: self.keys_released.clone(), 
            keys_down: self.keys_down.clone(), 
            mouse_delta: self.mouse_delta, 
            mouse_position: self.current_mouse_position,
            mouse_buttons_pressed: self.mouse_buttons_pressed.clone(), 
            mouse_buttons_released: self.mouse_buttons_released.clone(), 
            mouse_buttons_down: self.mouse_buttons_down.clone(), 
            mouse_scroll_delta: self.mouse_scroll_delta, 
            window_size: self.window_size,
            delta_time
        }
    }
}

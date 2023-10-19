#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{vec4, Vec4};

#[spirv(vertex)]
pub fn vs_main(
    input_color: Vec4,
    input_intensity: f32,
    color: &mut Vec4,
    #[spirv(position)] out_pos: &mut Vec4, 
    #[spirv(vertex_index)] index: u32
)
{
    let x = (1 - index as i32) as f32 * 0.5;
    let y = ((index & 1) as i32 * 2 - 1) as f32 * 0.5;
    *out_pos = vec4(x, y, 0.0, 1.0);

    *color = input_color * input_intensity;
}

#[spirv(fragment)]
pub fn fs_main(
    color: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] _color: &Vec4, 
    output: &mut Vec4) 
{
    *output = color;
}
#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{vec4, Vec4};

#[spirv(vertex)]
pub fn vs_main(#[spirv(position)] out_pos: &mut Vec4, #[spirv(vertex_index)] index: u32)
{
    let x = (1 - index as i32) as f32 * 0.5;
    let y = ((index & 1) as i32 * 2 - 1) as f32 * 0.5;
    *out_pos = vec4(x, y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn fs_main(output: &mut Vec4) 
{
    *output = vec4(0.3, 0.2, 0.1, 1.0);
}
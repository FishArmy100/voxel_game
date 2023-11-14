/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

mod utils;

use spirv_std::{
    glam::{self, UVec3},
    spirv, Image,
};

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, format = rgba8, sampled = false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] width_buffer: &u32,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] height_buffer: &u32,
) 
{
    let r = id.x as f32 / *width_buffer as f32;
    let g = id.y as f32 / *height_buffer as f32;

    unsafe { texture.write(id.truncate(), glam::vec4(r, g, 0.0, 1.0)) }
}

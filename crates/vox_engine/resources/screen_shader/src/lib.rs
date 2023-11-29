/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

use spirv_std::{
    glam::{self, Vec2, Vec4},
    image::Image2d,
    spirv, Sampler,
};

#[spirv(vertex)]
pub fn vs_main(
    out_uv: &mut Vec2,
    #[spirv(vertex_index)] vertex_id: i32,
    #[spirv(position)] position: &mut Vec4,
) {
    let x = (((vertex_id as u32 + 2) / 3) % 2) as f32;
    let y = (((vertex_id as u32 + 1) / 3) % 2) as f32;

    *out_uv = glam::vec2(x, y);
    *position = glam::vec4(-1.0 + x * 2.0, -1.0 + y * 2.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn fs_main(
    uv: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] screen_texture: &Image2d,
    #[spirv(descriptor_set = 0, binding = 1)] screen_texture_sampler: &Sampler,
    output: &mut Vec4,
) {
    *output = screen_texture.sample(*screen_texture_sampler, uv);
}

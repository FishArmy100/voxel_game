#![no_std]

use spirv_std::spirv;
use spirv_std::image::Image;
use spirv_std::glam::{vec4, Vec4, UVec3};

#[spirv(compute(threads(1)))]
pub fn main(
    #[spirv(descriptor_set = 0, binding = 0)] image: &Image!(2D, format=rgba32f, sampled=false),
    #[spirv(global_invocation_id)] id: UVec3,
)
{
    unsafe {
        image.write(id.truncate(), Vec4::new(1.0, 0.0, 0.0, 1.0))
    }
}
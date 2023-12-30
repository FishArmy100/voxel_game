#![no_std]
mod noise;

use noise::perlin_noise_3d;
use spirv_std::spirv;
use vox_core::{glam::{UVec3, IVec4}, terrain::TerrainArgs, utils::flatten_index};

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(global_invocation_id)] global_invocation_id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] voxels: &mut [u32],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] args: &TerrainArgs,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] chunk_index: &IVec4
)
{
    let index = flatten_index(global_invocation_id, UVec3::splat(args.chunk_size)) as usize;
    let pos = global_invocation_id.as_vec3a() + 0.001 / 40.0;
    let noise = perlin_noise_3d(pos);

    if noise > 0.5
    {
        voxels[index] = 1;
    }
    else 
    {
        voxels[index] = 2;
    }    
}

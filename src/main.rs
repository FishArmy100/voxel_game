use math::Vec3;
use voxel::brick_map::BrickMap;

mod math;
mod colors;
mod texture;
mod camera;
mod application;
mod rendering;
mod voxel;
mod debug_utils;
mod utils;
pub mod gpu;


fn main() 
{
    env_logger::init();
    pollster::block_on(application::run());
}
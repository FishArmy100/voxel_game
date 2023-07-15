mod math;
mod colors;
mod texture;
mod camera;
mod application;
mod rendering;
mod world;
mod voxel;
mod debug_utils;
fn main() 
{
    env_logger::init();
    pollster::block_on(application::run());
}
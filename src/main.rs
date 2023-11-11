mod math;
mod camera;
mod application;
mod rendering;
mod utils;
mod gpu_utils;
pub mod voxel;


fn main() 
{
    env_logger::init();
    pollster::block_on(application::run());
}
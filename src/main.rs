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
    
    let mut brick_map = BrickMap::<u32>::new(2, 1, None);
    brick_map.insert(Vec3::new(1, 3, 2), Some(1));
    println!("\n\nBrick map value: {:?}", brick_map.get(Vec3::new(1, 3, 2)));

    pollster::block_on(application::run());
}
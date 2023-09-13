use voxel::octree::Octree;

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
    // let mut octree = Octree::new(2);
    // octree.insert([0, 0, 0].into(), Some(8));
    // println!("{:?}", octree.get([1, 1, 0].into()));

    pollster::block_on(application::run());
}
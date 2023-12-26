use std::ops::{IndexMut, Index};
use std::time::SystemTime;

use glam::{U64Vec3, Vec4};

pub fn index_3d_to_index_1d(width: usize, height: usize, depth: usize, position: U64Vec3) -> u64
{
    (position.z as u64 * width as u64 * height as u64) + (position.y as u64 * width as u64) + position.x as u64
}

pub fn index_1d_to_index_3d(width: u64, height: u64, depth: u64, mut index: u64) -> U64Vec3
{
    let z = index / (width * height);
    index -= z * width * height;
    let y = index / width;
    let x = index % width;
    U64Vec3::new(x, y, z)
}

pub fn is_power_of_2(num: usize) -> bool 
{
    (num != 0) && ((num & (num - 1)) == 0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Array3D<T>
{
    width: usize,
    height: usize,
    depth: usize,

    data: Box<[T]>
}

impl<T> Array3D<T>
{
    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn depth(&self) -> usize { self.depth }

    pub fn new<F>(width: usize, height: usize, depth: usize, mut gen: F) -> Self
        where F : FnMut(usize, usize, usize) -> T
    {
        let mut data = Vec::with_capacity(width * height * depth);
        for z in 0..depth
        {
            for y in 0..height
            {
                for x in 0..width
                {
                    data.push(gen(x, y, z))
                }
            }
        }

        Self { width, height, depth, data: data.into_boxed_slice() }
    }

    pub fn from_vec(width: usize, height: usize, depth: usize, data: Vec<T>) -> Self
    {
        assert!(width * height * depth == data.len(), "data is not of the appropriate length.");
        Self 
        { 
            width, 
            height, 
            depth, 
            data: data.into_boxed_slice()
        }
    }

    pub fn as_slice(&self) -> &[T] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut [T] { &mut self.data }

    pub fn get(&self, position: U64Vec3) -> &T
    {
        assert!(position.x < self.width as u64 && position.y < self.height as u64 && position.z < self.depth as u64, "Index is out of range {:?}", position);

        let index = index_3d_to_index_1d(self.width, self.height, self.depth, position);
        &self.data[index as usize]
    }

    pub fn get_mut(&mut self, position: U64Vec3) -> &mut T
    {
        assert!(position.x < self.width as u64 && position.y < self.height as u64 && position.z < self.depth as u64, "Index is out of range {:?}", position);

        let index = index_3d_to_index_1d(self.width, self.height, self.depth, position);
        &mut self.data[index as usize]
    }
}

impl<T> Array3D<T> where T : Clone
{
    pub fn new_with_value(width: usize, height: usize, depth: usize, value: T) -> Self
    {
        Self::from_vec(width, height, depth, vec![value; width * height * depth])
    }
}

impl<T> Index<U64Vec3> for Array3D<T>
{
    type Output = T;

    fn index(&self, index: U64Vec3) -> &Self::Output {
        self.get(index)
    }
}

impl<T> Index<(usize, usize, usize)> for Array3D<T>
{
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        self.get(U64Vec3::new(index.0 as u64, index.1 as u64, index.2 as u64))
    }
}

impl<T> IndexMut<(usize, usize, usize)> for Array3D<T>
{
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        self.get_mut(U64Vec3::new(index.0 as u64, index.1 as u64, index.2 as u64))
    }
}

impl<T> IndexMut<U64Vec3> for Array3D<T>
{
    fn index_mut(&mut self, index: U64Vec3) -> &mut Self::Output {
        self.get_mut(index)
    }
}

/// Replaces the given option with `None`, and returns the old value.
/// Panics if `value` is none
pub fn replace_option<T>(value: &mut Option<T>) -> T
{
    let Some(value) = std::mem::replace(value, None) else {
        panic!("Tried to replace a None option");
    };

    value
}

pub unsafe trait Byteable : bytemuck::Pod + bytemuck::Zeroable {}
unsafe impl<T> Byteable for T where T : bytemuck::Pod + bytemuck::Zeroable {}

pub unsafe trait Wrappable : Copy {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Wrapper<T>(pub T) where T : Wrappable;

unsafe impl<T> bytemuck::Pod for Wrapper<T> where T : Wrappable + 'static {}
unsafe impl<T> bytemuck::Zeroable for Wrapper<T> where T : Wrappable {}

pub fn time_call<F, R>(f: F, name: &str) -> R
    where F : FnOnce() -> R
{
    let current_time = SystemTime::now();
    let r = f();
    let time = current_time.elapsed().unwrap().as_secs_f32() * 1000.0;
    println!("test '{}' took {}ms", name, time);
    r
}


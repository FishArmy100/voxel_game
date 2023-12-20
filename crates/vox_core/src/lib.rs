#![no_std]

pub mod camera;
pub mod utils;

pub use glam;
pub use num_traits::Float;
use glam::{f32::Vec3A, IVec3, Vec3, vec3a, uvec3, ivec3, UVec3};

use crate::utils::flatten_index;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ray 
{
    pub origin: Vec3A,
    pub dir: Vec3A
}

impl Ray 
{
    pub fn new<T>(origin: T, dir: T) -> Self 
        where T : Into<Vec3A>
    {
        Self 
        { 
            origin: origin.into(), 
            dir: dir.into(),
        }
    }

    pub fn from_points<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        Self 
        {
            origin: origin.into(),
            dir: origin.into() - destination.into()
        }
    }

    pub fn from_points_normalized<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        let dir: Vec3A = origin.into() - destination.into();
        Self 
        {
            origin: origin.into(),
            dir: dir.normalize()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HitInfo 
{
    pub hit: bool,
    pub hit_pos: Vec3A,
    pub distance: f32,
}

pub trait Intersectable 
{
    fn intersect(&self, ray: &Ray) -> HitInfo;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AABB
{
    pub min: Vec3A,
    pub max: Vec3A,
}

impl AABB 
{
    pub fn new<T>(min: T, max: T) -> Self
        where T : Into<Vec3A>
    {
        Self
        {
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn from_extents<T>(pos: T, extents: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        let min = pos.into() - extents.into();
        let max = pos.into() + extents.into();

        Self 
        {
            min,
            max,
        }
    }
}

impl Intersectable for AABB
{
    fn intersect(&self, ray: &Ray) -> HitInfo 
    {
        let t_min = (self.min - ray.origin) / ray.dir;
        let t_max = (self.max - ray.origin) / ray.dir;

        let t1 = t_min.min(t_max);
        let t2 = t_min.max(t_max);
        let near = t1.max_element();
        let far = t2.min_element();

        HitInfo 
        { 
            hit: !(near > far) && far >= 0.0, 
            hit_pos: ray.origin + ray.dir * near,
            distance: near
        }
    }
}

#[repr(C)]
pub struct VoxelModelHit
{
    pub hit: bool,
    pub value: u32,
    pub distance: f32,
}

impl VoxelModelHit
{
    pub const NONE: Self = Self { hit: false, value: 0, distance: 0.0 };

    pub const fn hit(value: u32, distance: f32) -> Self
    {
        Self 
        {
            hit: true,
            value,
            distance
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VoxelModel
{
    dim_x: u32,
    dim_y: u32,
    dim_z: u32,
    data_ptr: u32, 
}

impl VoxelModel
{
    pub fn dim_x(&self) -> u32 { self.dim_x }
    pub fn dim_y(&self) -> u32 { self.dim_y }
    pub fn dim_z(&self) -> u32 { self.dim_z }
    pub fn data_ptr(&self) -> u32 { self.data_ptr }

    pub fn num_voxels(&self) -> u32 { self.dim_x() * self.dim_y() * self.dim_z() }

    pub fn new(dim_x: u32, dim_y: u32, dim_z: u32, data_ptr: u32) -> Self
    {
        Self 
        {
            dim_x,
            dim_y,
            dim_z,
            data_ptr
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VoxelModelInstance
{
    model: VoxelModel,
    aabb: AABB,
    voxel_size: f32,
    origin: Vec3A
}

impl VoxelModelInstance
{
    pub fn dim_x(&self) -> u32 { self.model.dim_x() }
    pub fn dim_y(&self) -> u32 { self.model.dim_y() }
    pub fn dim_z(&self) -> u32 { self.model.dim_z() }
    pub fn data_ptr(&self) -> u32 { self.model.data_ptr() }

    pub fn voxel_size(&self) -> f32 { self.voxel_size }
    pub fn origin(&self) -> Vec3A { self.origin }
    pub fn aabb(&self) -> AABB { self.aabb }

    pub fn num_voxels(&self) -> u32 { self.model.num_voxels() }

    pub fn new<T>(origin: T, voxel_size: f32, model: VoxelModel) -> Self
        where T : Into<Vec3A> + Copy
    {
        let aabb = AABB {
            min: origin.into(),
            max: origin.into() + vec3a(model.dim_x as f32, model.dim_y as f32, model.dim_z as f32) * voxel_size
        };

        Self 
        {
            model,
            aabb,
            voxel_size,
            origin: origin.into()
        }
    }
    
    pub fn intersect(&self, ray: &Ray, voxels: &[u32]) -> VoxelModelHit
    {
        let hit = self.aabb.intersect(ray);
        if hit.hit
        {
            let ray = Ray {
                origin: hit.hit_pos - ray.dir * 0.01,
                dir: ray.dir
            };

            self.dda_intersect(&ray, hit.distance, voxels)
        }
        else 
        {
            VoxelModelHit::NONE
        }
    }

    fn dda_intersect(&self, ray: &Ray, ray_dist: f32, voxels: &[u32]) -> VoxelModelHit
    {
        let max_steps = self.dim_x() + self.dim_y() + self.dim_z() + 1; // TODO: compute at runtime

        let relative_origin = (ray.origin - self.origin) / self.voxel_size;

        let mut map_pos = relative_origin.floor().as_ivec3();
        let delta_dist = (ray.dir.length() / ray.dir).abs();
        
        let ray_step = ray.dir.signum().as_ivec3();

        let mut side_dist = (ray_step.as_vec3a() * (map_pos.as_vec3a() - relative_origin) + (ray_step.as_vec3a() * 0.5) + 0.5) * delta_dist;
        let mut i = 0;
        loop
        {
            if i == max_steps { break VoxelModelHit::NONE; }
            
            i += 1;

            let mask_x = if side_dist.x < side_dist.y.min(side_dist.z) { 1 } else { 0 };
            let mask_y = if side_dist.y < side_dist.z.min(side_dist.x) { 1 } else { 0 };
            let mask_z = if side_dist.z < side_dist.x.min(side_dist.y) { 1 } else { 0 };
            let mask = IVec3::new(mask_x, mask_y, mask_z);

            side_dist += mask.as_vec3a() * delta_dist;
            map_pos += mask * ray_step;

            if  map_pos.x < 0 || map_pos.x >= self.dim_x() as i32 ||
                map_pos.y < 0 || map_pos.y >= self.dim_y() as i32 ||
                map_pos.z < 0 || map_pos.z >= self.dim_z() as i32
            {
                continue;
            }

            let index = flatten_index(map_pos.as_uvec3(), uvec3(self.dim_x(), self.dim_y(), self.dim_z()));
            let value = voxels[(index + self.data_ptr()) as usize];
            if value != 0
            {
                let hit_pos = ((side_dist - ray.origin) + 0.5 - (ray_step.as_vec3a())) * delta_dist;
                let dda_distance = (ray.origin - hit_pos).length();

                break VoxelModelHit::hit(value, dda_distance + ray_dist);
            }
        }
    }
}
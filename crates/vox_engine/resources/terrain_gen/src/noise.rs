// See https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39

use spirv_std::num_traits::Float;
use vox_core::glam::{vec2, Vec2, Vec4, vec4, Vec3A, Vec3Swizzles, vec3a};
use vox_core::glam::Vec4Swizzles;


pub fn permute4(x: Vec4) -> Vec4 
{ 
    ((x * 34.0 + 1.0) * x) % Vec4::splat(289.0)
}

pub fn taylor_inverse_sqrt4(r: Vec4) -> Vec4
{
    1.79284291400159 - 0.85373472095314 * r // huh
}


pub fn fade3(t: Vec3A) -> Vec3A 
{
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn step(edge: Vec4, x: Vec4) -> Vec4
{
    vec4((edge.x < x.x) as i32 as f32, (edge.y < x.y) as i32 as f32, (edge.z < x.z) as i32 as f32, (edge.w < x.w) as i32 as f32)
}

pub fn mix4(a: Vec4, b: Vec4, v: f32) -> Vec4
{
    a * (1.0 - v) + b * v
}

pub fn mix2(a: Vec2, b: Vec2, v: f32) -> Vec2
{
    a * (1.0 - v) + b * v
}

pub fn mix(a: f32, b: f32, v: f32) -> f32
{
    a * (1.0 - v) + b * v
}


pub fn perlin_noise_3d(p: Vec3A) -> f32 
{
    let mut pi_0 = p.floor(); // Integer part for indexing
    let mut pi_1 = pi_0 + Vec3A::splat(1.); // Integer part + 1
    pi_0 = pi_0 % Vec3A::splat(289.);
    pi_1 = pi_1 % Vec3A::splat(289.);
    let pf_0 = p.fract(); // Fractional part for interpolation
    let pf_1 = pf_0 - Vec3A::splat(1.); // Fractional part - 1.
    let ix = vec4(pi_0.x, pi_1.x, pi_0.x, pi_1.x);
    let iy = vec4(pi_0.y, pi_0.y, pi_1.y, pi_1.y);
    let iz0 = pi_0.zzzz();
    let iz1 = pi_1.zzzz();

    let ixy = permute4(permute4(ix) + iy);
    let ixy0 = permute4(ixy + iz0);
    let ixy1 = permute4(ixy + iz1);

    let mut gx0 = ixy0 / 7.;
    let mut gy0 = (gx0.floor() / 7.).fract() - 0.5;
    gx0 = gx0.fract();
    let gz0 = Vec4::splat(0.5) - gx0.abs() - gy0.abs();
    let sz0 = step(gz0, Vec4::splat(0.));
    gx0 = gx0 + sz0 * (step(Vec4::splat(0.), gx0) - 0.5);
    gy0 = gy0 + sz0 * (step(Vec4::splat(0.), gy0) - 0.5);

    let mut gx1 = ixy1 / 7.;
    let mut gy1 = (gx1.floor() / 7.).fract() - 0.5;
    gx1 = gx1.fract();
    let gz1 = Vec4::splat(0.5) - gx1.abs() - gy1.abs();
    let sz1 = step(gz1, Vec4::splat(0.));
    gx1 = gx1 - sz1 * (step(Vec4::splat(0.), gx1) - 0.5);
    gy1 = gy1 - sz1 * (step(Vec4::splat(0.), gy1) - 0.5);

    let mut g000 = vec3a(gx0.x, gy0.x, gz0.x);
    let mut g100 = vec3a(gx0.y, gy0.y, gz0.y);
    let mut g010 = vec3a(gx0.z, gy0.z, gz0.z);
    let mut g110 = vec3a(gx0.w, gy0.w, gz0.w);
    let mut g001 = vec3a(gx1.x, gy1.x, gz1.x);
    let mut g101 = vec3a(gx1.y, gy1.y, gz1.y);
    let mut g011 = vec3a(gx1.z, gy1.z, gz1.z);
    let mut g111 = vec3a(gx1.w, gy1.w, gz1.w);

    let norm0 = taylor_inverse_sqrt4(vec4(g000.dot(g000), g010.dot(g010), g100.dot(g100), g110.dot(g110)));

    g000 = g000 * norm0.x;
    g010 = g010 * norm0.y;
    g100 = g100 * norm0.z;
    g110 = g110 * norm0.w;

    let norm1 = taylor_inverse_sqrt4(vec4(g001.dot(g001), g011.dot(g011), g101.dot(g101), g111.dot(g111)));

    g001 = g001 * norm1.x;
    g011 = g011 * norm1.y;
    g101 = g101 * norm1.z;
    g111 = g111 * norm1.w;

    let n000 = g000.dot(pf_0);
    let n100 = g100.dot(vec3a(pf_1.x, pf_0.y, pf_0.z));
    let n010 = g010.dot(vec3a(pf_0.x, pf_1.y, pf_0.z));
    let n110 = g110.dot(vec3a(pf_1.x, pf_1.y, pf_0.z));
    let n001 = g001.dot(vec3a(pf_0.x, pf_0.y, pf_1.z));
    let n101 = g101.dot(vec3a(pf_1.x, pf_0.y, pf_1.z));
    let n011 = g011.dot(vec3a(pf_0.x, pf_1.y, pf_1.z));
    let n111 = g111.dot(pf_1);

    let fade_xyz = fade3(pf_0);
    let n_z = mix4(vec4(n000, n100, n010, n110), vec4(n001, n101, n011, n111), fade_xyz.z);
    let n_yz = mix2(n_z.xy(), n_z.zw(), fade_xyz.y);
    let n_xyz = mix(n_yz.x, n_yz.y, fade_xyz.x);
    return 2.2 * n_xyz;
}


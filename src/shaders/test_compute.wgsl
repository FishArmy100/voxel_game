@group(0) @binding(0)
var<storage, read_write> v_indices: array<i32>; // this is used as both input and output for convenience

@group(0) @binding(1)
var<uniform> chunk_size: vec3<u32>;

@group(0) @binding(2)
var<uniform> chunk_pos: vec3<i32>;

const VOXEL_SIZE: f32 = 0.0625;
const EPSILON: f32 = 0.00000001;
const NOISE_HEIGHT_SCALE: f32 = 4.0;
const NOISE_HEIGHT_OFFSET: f32 = 1.0;
const NOISE_SCALE: f32 = 10.0;

const WATER_HEIGHT: f32 = 2.0;
const SAND_HEIGHT: f32 = 2.5;

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplexNoise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289(i); // Avoid truncation effects in permutation

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

fn sample_noise(x: u32, y: u32, z: u32) -> i32
{
    let chunk_offset = vec3<f32>(f32(chunk_pos.x) * f32(chunk_size.x), f32(chunk_pos.y) * f32(chunk_size.y), f32(chunk_pos.z) * f32(chunk_size.z));
    let pos = vec2<f32>((f32(x) + chunk_offset.x + EPSILON) * VOXEL_SIZE, (f32(z) + chunk_offset.z + EPSILON) * VOXEL_SIZE);
    let noise_height = simplexNoise2(pos / NOISE_SCALE) * NOISE_HEIGHT_SCALE + NOISE_HEIGHT_OFFSET;
    let voxel_height = (f32(y) + chunk_offset.y) * VOXEL_SIZE;

    var voxel = select(select(3, 2, voxel_height < SAND_HEIGHT), -1, voxel_height >= noise_height);
    voxel = select(voxel, 1, voxel == -1 && voxel_height < WATER_HEIGHT);

    return voxel;
}

fn index_of(x: u32, y: u32, z: u32) -> u32
{
    return (z * chunk_size.x * chunk_size.y) + (y * chunk_size.z) + x;
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) 
{
    let index = index_of(global_id.x, global_id.y, global_id.z);
    v_indices[index] = sample_noise(global_id.x, global_id.y, global_id.z);
}
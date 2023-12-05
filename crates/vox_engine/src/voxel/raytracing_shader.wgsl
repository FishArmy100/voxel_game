@group(0) @binding(0)
var output: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var<uniform> width: u32;

@group(0) @binding(2)
var<uniform> height: u32;

@group(0) @binding(3)
var<uniform> eye: vec4<f32>;

@group(0) @binding(4)
var<uniform> view_target: vec4<f32>;

@group(0) @binding(5)
var<uniform> fov: f32;

const VOXEL_COLOR: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);
const BACKGROUND_COLOR: vec4<f32> = vec4<f32>(0.5, 0.5, 0.5, 1.0);

struct Ray
{
    origin: vec3<f32>,
    dir: vec3<f32>
}

struct Camera
{
    eye: vec3<f32>,
    view_target: vec3<f32>,
    fov: f32
}

fn create_ray(x: u32, y: u32, camera: Camera) -> Ray 
{
    let aspect = f32(width) / f32(height);
    let theta = radians(camera.fov);
    let half_height = tan(theta / 2.0);
    let half_width = aspect * half_height;

    let w = normalize(camera.eye - camera.view_target);
    let u = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), w));
    let v = cross(w, u);

    let origin = camera.eye;
    let lower_left_corner = origin - (u * half_width) - (v * half_height) - w;
    let horizontal = u * 2.0 * half_width;
    let vertical = v * 2.0 * half_height;

    let xu = f32(x) / f32(width);
    let yv = f32(y) / f32(height);
    let dir = normalize(lower_left_corner + (horizontal * xu) + (vertical * yv) - origin);


    return Ray(origin, dir);
}

fn get_voxel(pos: vec3<f32>) -> bool
{
    let fpos = floor(pos);
    return (fpos.x == 0.0) && (fpos.y == 0.0) && (fpos.z == 0.0);
}

fn intersect_voxel(ray: Ray) -> bool
{
    let MAX_RAY_STEPS: u32 = 64u;

    var map_pos = floor(ray.origin);
    let delta_dist = abs(length(ray.dir) / ray.dir);

    
    let ray_step = sign(ray.dir);

    var side_dist = (ray_step * (map_pos - ray.origin) + (ray_step * 0.5) + 0.5) * delta_dist;

    var mask = vec3<bool>(false, false, false);
    var found = false;
    for (var i = 0u; i < MAX_RAY_STEPS; i += 1u)
    {
        found = found || get_voxel(map_pos);
        
        let yzx = vec3<f32>(side_dist.y, side_dist.z, side_dist.x);
        let zxy = vec3<f32>(side_dist.z, side_dist.x, side_dist.y);
        let m = min(yzx, zxy);
        mask = vec3<bool>(side_dist.x <= m.x, side_dist.y <= m.y, side_dist.z <= m.z);

        let v_mask = vec3<f32>(f32(u32(mask.x)), f32(u32(mask.y)), f32(u32(mask.z)));
        side_dist += v_mask * delta_dist;
        map_pos += v_mask * ray_step;
    }

    return found;
}

@compute @workgroup_size(1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) 
{
    let camera = Camera(eye.xyz, view_target.xyz, fov);
    let ray = create_ray(global_id.x, global_id.y, camera);
    let hit_voxel = intersect_voxel(ray);

    if (hit_voxel)
    {
        let voxel_color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
        textureStore(output, vec2<i32>(i32(global_id.x), i32(global_id.y)), voxel_color);
    }
    else 
    {
        let background_color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
        textureStore(output, vec2<i32>(i32(global_id.x), i32(global_id.y)), background_color);
    }
}
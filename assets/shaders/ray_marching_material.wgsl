struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    aspect_ratio: f32,
    max_steps: u32,
    min_dist: f32,
    max_dist: f32,
    tan_fov: f32,
};

struct Scene {
    spheres: array<Sphere>,
}

struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    time: f32,
    // The delta time since the previous frame in seconds
    delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: f32
#endif
}

@group(0) @binding(1)
var<uniform> globals: Globals;

@group(2) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(1)
var<storage, read> scene: Scene;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(vertex.position, 1.0);
    out.uv_coords = (vertex.uv_coords * 2.0 - 1.0) * camera.tan_fov;
    out.uv_coords.x *= camera.aspect_ratio;
    return out;
}

struct FragmentIn {
    @location(0) uv_coords: vec2<f32>,
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c: f32 = v * s;
    let x: f32 = c * (1.0 - abs((h / 60.0) % 2.0 - 1.0));
    let m: f32 = v - c;
    var rgb: vec3<f32>;

    if (h < 60.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 120.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 180.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 240.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 300.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}

fn dist(p1: vec4<f32>, p2: vec4<f32>) -> f32 {
    let d = p1 - p2;
    return sqrt(dot(d, d));
}

fn march_ray(origin: vec4<f32>, direction: vec4<f32>, distance: f32) -> vec4<f32> {
    return origin + distance * direction;
}

struct Sphere {
    centre: vec4<f32>,
    radius: f32,
    material_id: u32,
}

struct SDFResult {
    distance: f32,
    material_id: u32,
}

fn sphere_sdf(sphere: Sphere, p: vec4<f32>) -> f32 {
    return dist(p, sphere.centre) - sphere.radius;
}

fn scene_sdf(p: vec4<f32>) -> SDFResult {
    var min_dist: f32 = 1000.0;
    var current_dist: f32;
    var material_id: u32 = 0;
    var sphere: Sphere;
    for (var i: u32 = 0; i < arrayLength(&scene.spheres); i++) {
        sphere = scene.spheres[i];
        current_dist = sphere_sdf(sphere, p);
        if current_dist < min_dist {
            min_dist = current_dist;
            material_id = sphere.material_id;
        }
    }
    if p.y < min_dist {
        material_id = 2u;
        min_dist = p.y;
    }

    return SDFResult(min_dist, material_id);
}

fn material_to_col(material_id: u32) -> vec4<f32> {
    if material_id == 2 {
        return vec4(0.7, 0.1, 0.3, 1.0);
    }
    return vec4(0.3, 0.4, 0.5, 1.0);
}

fn ray_march(ray_origin: vec4<f32>, ray_direction: vec4<f32>) -> vec4<f32> {
    var dist: f32 = 0.0;
    var current_pos: vec4<f32> = ray_origin;
    var current_sdf: SDFResult;
    for (var i: u32 = 0; i < camera.max_steps; i++) {
        current_sdf = scene_sdf(current_pos);
        
        if current_sdf.distance < 0 {
            return material_to_col(current_sdf.material_id);
        }

        dist += max(current_sdf.distance, camera.min_dist);

        if dist >= camera.max_dist {
            return vec4(1.0, 0.0, 1.0, 1.0);
        }

        current_pos = march_ray(ray_origin, ray_direction, dist);
    }

    return vec4(vec3(0.0), 1.0);
}

@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {
    var camera_origin = camera.position;
    var ray_origin = camera_origin + camera.forward * 1.0 + (in.uv_coords.x * camera.right) + (in.uv_coords.y * camera.up);
    var ray_direction = normalize(ray_origin - camera_origin);

    var color = ray_march(ray_origin, ray_direction);

    return vec4(color.x, color.y, color.z * 0.1 * f32(arrayLength(&scene.spheres)), 1.0);
}

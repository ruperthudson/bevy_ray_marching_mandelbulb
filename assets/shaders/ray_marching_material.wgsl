struct Camera {
    position: vec3<f32>,
    forward: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    aspect_ratio: f32,
    power: f32,
    max_iterations: u32,
    bailout: f32,
    num_steps: u32,
    min_dist: f32,
    max_dist: f32,
    zoom: f32,
};

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

@group(1) @binding(0)
var<uniform> camera: Camera;

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
    out.uv_coords = (vertex.uv_coords * 2.0 - 1.0) / camera.zoom;
    out.uv_coords.x *= camera.aspect_ratio;
    return out;
}

struct FragmentIn {
    @location(0) uv_coords: vec2<f32>,
}

struct MandelbulbResult {
    de: f32,         // Distance Estimator value
    iterations: u32, // Number of iterations
};

fn mandelbulb_de(position: vec3<f32>, power: f32, max_iterations: u32, bailout: f32) -> MandelbulbResult {
    var z = position;
    var dr = 1.0;
    var r = 0.0;
    var i: u32 = 0u;
    for (i = 0u; i < max_iterations; i = i + 1u) {
        r = length(z);
        if (r > bailout) {
            break;
        }

        // Convert to polar coordinates
        var theta = acos(z.z / r);
        var phi = atan2(z.y, z.x);
        dr =  pow(r, power - 1.0) * power * dr + 1.0;

        // Scale and rotate the point
        var zr = pow(r, power);
        theta = theta * power;
        phi = phi * power;

        // Convert back to Cartesian coordinates
        z = zr * vec3<f32>(sin(theta) * cos(phi), sin(phi) * sin(theta), cos(theta));
        z = z + position;
    }
    return MandelbulbResult(0.5 * log(r) * r / dr, i);
}

fn calculate_normal(current_position: vec3<f32>, power: f32, max_iterations: u32, bailout: f32) -> vec3<f32> {
    let SMALL_STEP = 0.001; // This value might need tweaking based on your scene scale
    let step_x = vec3<f32>(SMALL_STEP, 0.0, 0.0);
    let step_y = vec3<f32>(0.0, SMALL_STEP, 0.0);
    let step_z = vec3<f32>(0.0, 0.0, SMALL_STEP);

    // Here, we calculate the distance from the mandelbulb surface in each axis direction.
    // Note: the distance estimator function mandelbulb_de replaces get_distance_from_world.
    let gradient_x = mandelbulb_de(current_position + step_x, power, max_iterations, bailout).de 
                   - mandelbulb_de(current_position - step_x, power, max_iterations, bailout).de;
    let gradient_y = mandelbulb_de(current_position + step_y, power, max_iterations, bailout).de 
                   - mandelbulb_de(current_position - step_y, power, max_iterations, bailout).de;
    let gradient_z = mandelbulb_de(current_position + step_z, power, max_iterations, bailout).de 
                   - mandelbulb_de(current_position - step_z, power, max_iterations, bailout).de;

    // Construct the normal from the gradient components and normalize it
    let normal = vec3<f32>(gradient_x, gradient_y, gradient_z);
    return normalize(normal);
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

fn calculate_base_color(iterations: f32, maxIterations: f32) -> vec3<f32> {
    let normalized: f32 = iterations / maxIterations;

    // Hue
    let hue: f32 = normalized * 360.0;

    // Saturation (varying between 0.5 and 1.0 using a sine wave)
    let saturation: f32 = 0.5 + 0.5 * sin(3.14159 * 3.0 * normalized);

    // Value (varying between 0.8 and 1.0 using a sine wave for more brightness variation)
    let value: f32 = 0.8 + 0.2 * sin(3.14159 * 6.0 * normalized);

    // Convert HSV to RGB (assuming this function exists in your setup)
    let color: vec3<f32> = hsv_to_rgb(hue, saturation, value);

    return color;
}

fn ambient_occlusion(position: vec3<f32>, normal: vec3<f32>, power: f32, max_iterations: u32, bailout: f32) -> f32 {
    let NUM_SAMPLES: u32 = 8u; // Number of AO samples. Increase for better quality at the cost of performance.
    let AO_STEP: f32 = 0.05; // Step size for AO samples
    let MAX_AO_DISTANCE: f32 = 0.5; // Maximum distance to check for occlusion

    var ao_sum = 0.0;
    var ao_distance = AO_STEP;

    for (var i = 0u; i < NUM_SAMPLES; i += 1u) {
        var sample_position = position + normal * ao_distance;
        var sample_result = mandelbulb_de(sample_position, power, max_iterations, bailout);
        
        if (sample_result.de < AO_STEP) {
            ao_sum += (1.0 - sample_result.de / AO_STEP);
        }

        ao_distance += AO_STEP;
        if (ao_distance > MAX_AO_DISTANCE) {
            break;
        }
    }

    return 1.0 - ao_sum / f32(NUM_SAMPLES);
}

fn ray_march(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> vec3<f32> {
    var total_distance_traveled = 0.0;
    let NUMBER_OF_STEPS: u32 = camera.num_steps;
    let MINIMUM_HIT_DISTANCE: f32 = camera.min_dist;
    let MAXIMUM_TRAVEL_DISTANCE: f32 = camera.max_dist;

    // Mandelbulb specific parameters
    let power: f32 = camera.power;
    let max_iterations: u32 = camera.max_iterations;
    let bailout: f32 = camera.bailout;

    // Lighting parameters
    let light_position = vec3<f32>(2.0, -5.0, -3.0);

    // Downward-facing light parameters
    let down_light_position = vec3<f32>(0.0, 5.0, 0.0); // Position the light above the Mandelbulb
    let down_light_direction = normalize(vec3<f32>(0.0, -1.0, 0.0)); // Direction pointing downwards
    let down_light_intensity: f32 = 0.8; // Intensity of the downward-facing light
    let down_light_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0); // White color for the downward light
    // Ambient
    let ambient_light_intensity: f32 = 0.0001;
    let ambient_light_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let specular_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let shininess: f32 = 10.0;

    for (var i = 0u; i < NUMBER_OF_STEPS; i += 1u) {
        let current_position = ray_origin + total_distance_traveled * ray_direction;
        let result = mandelbulb_de(current_position, power, max_iterations, bailout);

        if result.de < MINIMUM_HIT_DISTANCE {
            let normal = calculate_normal(current_position, power, max_iterations, bailout);

            let direction_to_light = normalize(light_position - current_position);
            // Direction to the downward-facing light from the current position
            let direction_to_down_light = normalize(down_light_position - current_position);

            let ambient = ambient_light_color * ambient_light_intensity;
            let diffuse_intensity = max(0.0, dot(normal, direction_to_light));
            let base_color = calculate_base_color(f32(result.iterations), f32(max_iterations));
            let diffuse = base_color * diffuse_intensity;
            let view_direction = normalize(ray_origin - current_position);
            let reflect_direction = reflect(-direction_to_light, normal);
            let specular_factor = max(dot(view_direction, reflect_direction), 0.0);
            let specular_intensity = pow(specular_factor, shininess);
            let specular = specular_color * specular_intensity;

            // Downward-facing light calculations
            let down_light_diffuse_intensity = max(0.0, dot(normal, direction_to_down_light));
            let down_diffuse = base_color * down_light_diffuse_intensity;

            // Combine the two light sources (directional light + downward light)
            var final_color = ambient + diffuse + specular + down_diffuse;


            // Apply ambient occlusion
            let ao = ambient_occlusion(current_position, normal, power, max_iterations, bailout);
            final_color *= ao; // Multiply the final color by the AO value

            return final_color;
        }

        if total_distance_traveled > MAXIMUM_TRAVEL_DISTANCE {
            break; 
        }

        total_distance_traveled += result.de;
    } 

    return vec3<f32>(0.0, 0.0, 0.0);
}


@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {
    var camera_origin = camera.position;
    var ray_origin = camera_origin + camera.forward * 1.0 + (in.uv_coords.x * camera.horizontal) + (in.uv_coords.y * camera.vertical);
    var ray_direction = normalize(ray_origin - camera_origin);

    var color = ray_march(ray_origin, ray_direction);

    return vec4(color.x, color.y, color.z, 1.0);
}

struct Camera {
    position: vec3<f32>,
    forward: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    aspect_ratio: f32,
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
    out.uv_coords = vertex.uv_coords * 2.0 - 1.0;
    out.uv_coords.x *= camera.aspect_ratio;
    return out;
}

struct FragmentIn {
    @location(0) uv_coords: vec2<f32>,
}

fn get_distance_from_sphere(current_position: vec3<f32>, sphere_center: vec3<f32>, radius: f32) -> f32 {
    return length(current_position - sphere_center) - radius;
}

// Constant representing a very large number, akin to "infinity."
let VERY_LARGE_NUMBER: f32 = 1e30;

// Function to calculate the shortest distance between a point and a line segment.
fn point_to_line_segment_distance(pnt: vec3<f32>, end1: vec3<f32>, end2: vec3<f32>) -> f32 {
    var v = end2 - end1;
    var w = pnt - end1;

    var c1 = dot(w, v);
    if ( c1 <= 0.0 ) {
        return length(pnt - end1);
    }

    var c2 = dot(v, v);
    if ( c2 <= c1 ) {
        return length(pnt - end2);
    }

    var b = c1 / c2;
    var pb = end1 + b * v;
    return length(pnt - pb);
}

// helper function to compute the distance from a point to a triangle.
fn compute_distance_to_triangle(triangle_vertices: array<vec3<f32>, 3>, pnt: vec3<f32>) -> f32 {
    var edge1 = triangle_vertices[1] - triangle_vertices[0];
    var edge2 = triangle_vertices[2] - triangle_vertices[0];
    var normal = normalize(cross(edge1, edge2));

    var proj = pnt - dot(pnt - triangle_vertices[0], normal) * normal;
    var dist_to_plane = length(pnt - proj);

    // Check if the projected point is inside the triangle.
    var c0 = cross(edge1, proj - triangle_vertices[0]);
    var c1 = cross(triangle_vertices[2] - triangle_vertices[1], proj - triangle_vertices[1]);
    var c2 = cross(triangle_vertices[0] - triangle_vertices[2], proj - triangle_vertices[2]);

    if (dot(normal, c0) >= 0.0 && dot(normal, c1) >= 0.0 && dot(normal, c2) >= 0.0) {
        return dist_to_plane; // The point is inside the triangle.
    } else {
        // The point is outside the triangle, so we find the closest distance to the triangle's edges.
        var dist_edge1 = point_to_line_segment_distance(pnt, triangle_vertices[0], triangle_vertices[1]);
        var dist_edge2 = point_to_line_segment_distance(pnt, triangle_vertices[1], triangle_vertices[2]);
        var dist_edge3 = point_to_line_segment_distance(pnt, triangle_vertices[2], triangle_vertices[0]);

        // Return the smallest distance.
        return min(dist_edge1, min(dist_edge2, dist_edge3));
    }
}

// Helper function to calculate the distance to a triangle.
fn distance_to_triangle(pnt: vec3<f32>, tri: array<vec3<f32>, 3>) -> f32 {
    return compute_distance_to_triangle(tri, pnt); // Assuming compute_distance_to_triangle is predefined.
}

// Main function to calculate the distance to a tetrahedron.
fn get_distance_to_tetrahedron(pnt: vec3<f32>, center: vec3<f32>) -> f32 {
    // Define the four vertices of the tetrahedron.
    let tetra_size = 1.0; // Define the size of the tetrahedron
    let height = sqrt(6.0) / 3.0 * tetra_size;
    let inv_sqrt_3 = 1.0 / sqrt(3.0);

    // Calculate the positions based on the center
    let v0 = center + vec3<f32>(0.0, -height * 2.0 / 3.0, 0.0);
    let v1 = center + vec3<f32>(-inv_sqrt_3 * tetra_size, -height / 3.0, -0.5 * tetra_size);
    let v2 = center + vec3<f32>(inv_sqrt_3 * tetra_size, -height / 3.0, -0.5 * tetra_size);
    let v3 = center + vec3<f32>(0.0, -height / 3.0, tetra_size);

    // Define the faces of the tetrahedron.
    let faces: array<array<vec3<f32>, 3>, 4> = array<array<vec3<f32>, 3>, 4>(
        array<vec3<f32>, 3>(v0, v1, v2),
        array<vec3<f32>, 3>(v0, v3, v1),
        array<vec3<f32>, 3>(v0, v2, v3),
        array<vec3<f32>, 3>(v1, v3, v2)
    );

    var min_distance = VERY_LARGE_NUMBER; // Start with a very high distance that will be lowered during the checks.

    // Manually unroll the loop and calculate the minimum distance.
    var distance = compute_distance_to_triangle(faces[0], pnt); // Replace with your actual distance function
    min_distance = min(min_distance, distance);

    distance = compute_distance_to_triangle(faces[1], pnt);
    min_distance = min(min_distance, distance);

    distance = compute_distance_to_triangle(faces[2], pnt);
    min_distance = min(min_distance, distance);

    distance = compute_distance_to_triangle(faces[3], pnt);
    min_distance = min(min_distance, distance);

    return min_distance;
}

fn get_distance_to_sierpinski_tetrahedron(pnt: vec3<f32>, center: vec3<f32>, iterations: u32) -> f32 {
    var new_pnt = pnt - center; // Translate point to origin
    var scale = 1.0;
    let scale_factor = 1.1; // This determines how much smaller each iteration is.

    for (var i: u32 = 0u; i < iterations; i = i + 1u) {
        // Calculate distance for each offset manually
        var d0 = length(new_pnt - scale * vec3<f32>(1.0, -1.0, 1.0));
        var d1 = length(new_pnt - scale * vec3<f32>(-1.0, -1.0, -1.0));
        var d2 = length(new_pnt - scale * vec3<f32>(1.0, -1.0, -1.0));
        var d3 = length(new_pnt - scale * vec3<f32>(-1.0, -1.0, 1.0));

        // Determine the minimum distance and corresponding offset
        var min_dist = d0;
        var target_offset = vec3<f32>(1.0, -1.0, 1.0);

        if (d1 < min_dist) {
            min_dist = d1;
            target_offset = vec3<f32>(-1.0, -1.0, -1.0);
        }
        if (d2 < min_dist) {
            min_dist = d2;
            target_offset = vec3<f32>(1.0, -1.0, -1.0);
        }
        if (d3 < min_dist) {
            min_dist = d3;
            target_offset = vec3<f32>(-1.0, -1.0, 1.0);
        }

        // Scale and translate the point into the space of the chosen sub-tetrahedron
        new_pnt = scale_factor * (new_pnt - scale * target_offset);

        // Update the scale for the next iteration
        scale = scale / scale_factor;
    }

    // After all iterations, use the tetrahedron distance function one last time
    return get_distance_to_tetrahedron(new_pnt, vec3<f32>(0.0, 0.0, 0.0)) * scale; // Multiplying by 'scale' accounts for the scaled space.
}

//fn get_distance_to_tetrahedron(pnt: vec3<f32>, center: vec3<f32>) -> f32 {
//    // Define the four vertices of the tetrahedron.
//    let tetra_size = 1.0; // define the size of the tetrahedron
//    let height = sqrt(6.0) / 3.0 * tetra_size;
//    let inv_sqrt_3 = 1.0 / sqrt(3.0);
//    
//    // calculate the positions based on the center
//    let v0 = center + vec3<f32>(0.0, -height * 2.0 / 3.0, 0.0);
//    let v1 = center + vec3<f32>(-inv_sqrt_3 * tetra_size, -height / 3.0, -0.5 * tetra_size);
//    let v2 = center + vec3<f32>(inv_sqrt_3 * tetra_size, -height / 3.0, -0.5 * tetra_size);
//    let v3 = center + vec3<f32>(0.0, -height / 3.0, tetra_size);
//
//    var min_distance = VERY_LARGE_NUMBER; // Start with a very high distance that will be lowered during the checks.
//
//    // Calculate the distance to each triangle in the tetrahedron
//    min_distance = min(min_distance, compute_distance_to_triangle(array<vec3<f32>, 3>(v0, v1, v2), pnt));
//    min_distance = min(min_distance, compute_distance_to_triangle(array<vec3<f32>, 3>(v0, v3, v1), pnt));
//    min_distance = min(min_distance, compute_distance_to_triangle(array<vec3<f32>, 3>(v0, v2, v3), pnt));
//    min_distance = min(min_distance, compute_distance_to_triangle(array<vec3<f32>, 3>(v1, v3, v2), pnt));
//
//    return min_distance;
//}

fn distance(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    return sqrt(dx*dx + dy*dy + dz*dz);
}

fn get_distance_from_world(current_position: vec3<f32>, num_tetrahedrons: f32) -> f32 {
    let tetra_spacing: f32 = 1.15; // The distance between the centers of tetrahedrons along the x-axis
    let tetra_height: f32 = 1.0;
    let y_offset: f32 = -tetra_height / 3.0;

    let initial_center: vec3<f32> = vec3<f32>(-0.65, 0.0, 1.0); // Center of the first tetrahedron

    var min_distance: f32 = 1.0e30; // Starting with a very large number for comparison

    for (var i: f32 = 0.0; i < num_tetrahedrons; i += 1.0) {
        let x_offset: f32 = i * tetra_spacing;

        // Calculate the center for the current tetrahedron
        let current_center: vec3<f32> = initial_center + vec3<f32>(x_offset, 0.0, 0.0);

        // Getting the distance to the Sierpinski tetrahedron instead of a regular tetrahedron
        var current_distance: f32 = get_distance_to_sierpinski_tetrahedron(current_position, current_center, 10u);
        
        min_distance = min(min_distance, current_distance);

        // If you have more tetrahedrons stacked or in different positions, you can
        // add more calls to the Sierpinski distance function here, similar to how
        // it was done for the regular tetrahedron. Just calculate the new center and
        // call the function again.
    }

    return min_distance;
}

//Calculate the normal for any shape by calculating the gradient
// We calculate the gradient by taking a small offset in each unit direction and find the difference
fn calculate_normal(current_position: vec3<f32>) -> vec3<f32> {
    var SMALL_STEP = vec2<f32>(0.001, 0.0);

    var gradient_x = get_distance_from_world(current_position + SMALL_STEP.xyy, 1.0) - get_distance_from_world(current_position - SMALL_STEP.xyy, 1.0);
    var gradient_y = get_distance_from_world(current_position + SMALL_STEP.yxy, 1.0) - get_distance_from_world(current_position - SMALL_STEP.yxy, 1.0);
    var gradient_z = get_distance_from_world(current_position + SMALL_STEP.yyx, 1.0) - get_distance_from_world(current_position - SMALL_STEP.yyx, 1.0);

    return normalize(vec3<f32>(gradient_x, gradient_y, gradient_z));
}

fn ray_march(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> vec3<f32> {
    var total_distance_traveled = 0.0;
    var NUMBER_OF_STEPS = 256;
    var MINIMUM_HIT_DISTANCE = 0.00001;
    var MAXIMUM_TRAVEL_DISTANCE = 100000.0;

    for(var i = 0; i < NUMBER_OF_STEPS; i++) {
        var current_position = ray_origin + total_distance_traveled * ray_direction;

        var distance_to_closest = get_distance_from_world(current_position, 1.0);

        if(distance_to_closest < MINIMUM_HIT_DISTANCE) {
            var normal = calculate_normal(current_position);

            var light_position = vec3<f32>(2.0, -5.0, -3.0);

            var direction_to_light = normalize(current_position - light_position);

            var diffuse_intensity = max(0.0, dot(normal, direction_to_light));

            return vec3<f32>(1.0, 0.0, 0.0) * diffuse_intensity;
        }

        if(total_distance_traveled > MAXIMUM_TRAVEL_DISTANCE) {
            //No hit has occured, break out of the loop
            break;
        }

        total_distance_traveled += distance_to_closest;
    } 

    //A miss has occured so return a background color
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

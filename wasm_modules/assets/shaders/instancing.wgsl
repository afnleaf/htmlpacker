// there are two shader stages in this file
// vertex shader, which i gather is the positional and mesh data
// where the shape goes in render world
// fragment shader, which i gather is colours and materials for each pixel

// a vertex is a point where two or more curves, lines, or edges meet
// so think the 3 vertices of a triangle
// shader runs for each vertex in a mesh. (each of the 8 points of a prism)
// a normal is a 3d lighting vector that points out from the surface of the mesh
// a uv is a 2d coordinate (x,y renamed to u,v) corresponding the a 3d point
// uv mapping is a process of telling the GPU how a 2d texture wraps a 3d mesh

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
// Import Bevy's lighting and view bindings
#import bevy_pbr::mesh_view_bindings::{lights, view}
#import bevy_pbr::lighting::{point_light, spot_light, directional_light}
#import bevy_pbr::mesh_view_bindings::globals


@group(2) @binding(0) var<storage, read> elevation_buffer: array<i32>;
// map_id, points_per_map, padding, padding
@group(2) @binding(1) var<uniform> map_selection: vec4<u32>;
// x, y, z, w
@group(2) @binding(2) var<uniform> sun_position: vec4<f32>;
// we need a sun position uniform buff for the fragment shader

// buffer goes in and buffer goes out

// layout of the incoming vertex data
struct Vertex {
    // from vertex buffer 0 (prism mesh)
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // from vertex buffer 1 (instance data)
    // we set these shader locations in custom pipeline specialize
    @location(3) i_pos_scale: vec4<f32>,  // xyz = position, w = scale
    @location(4) i_rotation: vec4<f32>,   // quaternion (xyzw)
    @location(5) i_color: vec4<f32>,
    @location(6) elevation_index: u32,
};

//layout of the outgoing vertex data
struct CustomVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};


// need to this apply the rotation in shader
fn apply_quaternion_rotation(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> { 
    // extract quaternion components (q = xi + yj + zk + w)
    let qvec = q.xyz;
    let qw = q.w;

    // apply quaternion rotation formula
    // where x = cross
    // v' = v + 2 * qvec x (qvec x v + w * v)
    let uv = cross(qvec, v);
    let uuv = cross(qvec, uv);
    return v + 2.0 * (uv * qw + uuv);
}

fn getColorFromElevation(elevation: f32) -> vec4<f32> {
    if (elevation < -6000.0) {
        return vec4<f32>(8.0 / 255.0, 14.0 / 255.0, 48.0 / 255.0, 1.0); // 0x080e30
    } else if (elevation < -3000.0) {
        return vec4<f32>(31.0 / 255.0, 45.0 / 255.0, 71.0 / 255.0, 1.0); // 0x1f2d47
    } else if (elevation < -150.0) {
        return vec4<f32>(42.0 / 255.0, 60.0 / 255.0, 99.0 / 255.0, 1.0); // 0x2a3c63
    } else if (elevation < -50.0) {
        return vec4<f32>(52.0 / 255.0, 75.0 / 255.0, 117.0 / 255.0, 1.0); // 0x344b75
    } else if (elevation < 0.0001) {
        return vec4<f32>(87.0 / 255.0, 120.0 / 255.0, 179.0 / 255.0, 1.0); // 0x5778b3
    } else if (elevation < 75.0) {
        return vec4<f32>(79.0 / 255.0, 166.0 / 255.0, 66.0 / 255.0, 1.0);  // 0x4fa642
    } else if (elevation < 150.0) {
        return vec4<f32>(52.0 / 255.0, 122.0 / 255.0, 42.0 / 255.0, 1.0);  // 0x347a2a
    } else if (elevation < 400.0) {
        return vec4<f32>(0.0 / 255.0, 83.0 / 255.0, 11.0 / 255.0, 1.0);   // 0x00530b
    } else if (elevation < 1000.0) {
        return vec4<f32>(61.0 / 255.0, 55.0 / 255.0, 4.0 / 255.0, 1.0);    // 0x3d3704
    } else if (elevation < 2000.0) {
        return vec4<f32>(128.0 / 255.0, 84.0 / 255.0, 17.0 / 255.0, 1.0);   // 0x805411
    } else if (elevation < 3200.0) {
        return vec4<f32>(151.0 / 255.0, 122.0 / 255.0, 68.0 / 255.0, 1.0);  // 0x977944
    } else if (elevation < 5000.0) {
        return vec4<f32>(182.0 / 255.0, 181.0 / 255.0, 181.0 / 255.0, 1.0); // 0xb6b5b5
    } else {
        return vec4<f32>(238.0 / 255.0, 238.0 / 255.0, 238.0 / 255.0, 1.0); // 0xeeeeee
    }
}

// first we get the prism's local vertex position (a point on the prism)
// apply the rotation to that vertex position
// apply the scale to the rotated position
// apply the translation (world position)
// use a bevy helper function to transform this final position into clip space
// this allows the gpu to know where on the screen to draw it
// transform normal with rotation
// transform normal to world space (assuming no non-uniform scaling)
// store world position for lighting calculations
// pass instance color and transformed normal to the fragment shader
// return out buffer

@vertex
fn vertex(vertex: Vertex) -> CustomVertexOutput {
    var out: CustomVertexOutput;

    let global_index = 
    map_selection.x * (map_selection.y) + vertex.elevation_index;
    //let elevation_meters = f32(elevation_buffer[global_index]) * 0.001;
    let elevation_meters = f32(elevation_buffer[global_index]);

    let base_sphere_position = vertex.i_pos_scale.xyz;
    let sphere_radius = length(base_sphere_position);
    let sphere_direction = normalize(base_sphere_position);

    let elevation_world_units = elevation_meters * 0.00004;
    let elevated_instance_pos = 
    sphere_direction * (sphere_radius + elevation_world_units);

    let rotated_position = 
    apply_quaternion_rotation(vertex.i_rotation, vertex.position);
    let scaled_position = rotated_position * vertex.i_pos_scale.w;

    let final_position = scaled_position + elevated_instance_pos;
    
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0u),
        vec4<f32>(final_position, 1.0)
    );
    
    let rotated_normal = 
    apply_quaternion_rotation(vertex.i_rotation, vertex.normal);
    
    let world_from_local = get_world_from_local(0u);
    out.world_normal = 
    normalize((world_from_local * vec4<f32>(rotated_normal, 0.0)).xyz);
    
    out.world_position = 
    (world_from_local * vec4<f32>(final_position, 1.0)).xyz;

    let color = getColorFromElevation(elevation_meters);
    
    //out.color = vertex.i_color;
    out.color = color;
    
    return out;
}


// this fragment shader runs for every pixel of every prism
// receives interpolated color and normal
// simple directional lighting, calculate diffuse lighting
// calculates simple lighting and returns final "lit" pixel color
// needs time or sun position to create moving effect
/*
@fragment
fn fragment(in: CustomVertexOutput) -> @location(0) vec4<f32> {
    // position here would be good
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let ambient = 0.3;
    
    let n_dot_l = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = n_dot_l * 0.7;
    
    let lit_color = in.color.rgb * (ambient + diffuse);
    
    return vec4<f32>(lit_color, in.color.a);
    //return vec4<f32>(in.color.rgb, in.color.a);
}
*/

@fragment
fn fragment(in: CustomVertexOutput) -> @location(0) vec4<f32> {
    // Calculate light direction from fragment to sun
    // Since sun is very far away (149,000 units), we can treat it as directional
    let light_dir = normalize(sun_position.xyz - in.world_position);
    
    // Alternative: For a true directional light (sun at infinity)
    // just use sun position as direction since Earth is at origin
    // let light_dir = normalize(sun_position.xyz);
    
    // Ambient light - base illumination even in shadow
    let ambient = 0.3;
    
    // Diffuse lighting - how directly the surface faces the sun
    let n_dot_l = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = n_dot_l * 0.7;
    
    // Combine ambient and diffuse lighting
    let lit_color = in.color.rgb * (ambient + diffuse);
    
    return vec4<f32>(lit_color, in.color.a);
}

/*
@fragment
fn fragment(in: CustomVertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(sun_position.xyz);
    
    // Enhanced ambient with subtle blue tint for atmosphere
    let ambient_color = vec3<f32>(0.25, 0.27, 0.35);
    let ambient = in.color.rgb * ambient_color;
    
    // Warmer sunlight color
    let sun_color = vec3<f32>(1.0, 0.95, 0.8);
    let n_dot_l = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = in.color.rgb * sun_color * n_dot_l * 0.75;
    
    // Add slight rim lighting for atmosphere effect
    let view_dir = normalize(-in.world_position); // camera at origin
    let rim = 1.0 - max(dot(view_dir, in.world_normal), 0.0);
    let rim_light = pow(rim, 3.0) * 0.1 * vec3<f32>(0.5, 0.7, 1.0);
    
    let final_color = ambient + diffuse + rim_light;
    
    return vec4<f32>(final_color, in.color.a);
}
*/

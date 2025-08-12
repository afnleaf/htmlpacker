// Complete instancing.wgsl shader file with sun integration

// Import required Bevy functions
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<storage, read> elevation_buffer: array<i32>;
@group(2) @binding(1) var<uniform> map_selection: vec4<u32>;

// Vertex input structure
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_rotation: vec4<f32>,
    @location(5) i_color: vec4<f32>,
    @location(6) elevation_index: u32,
};

// Vertex output structure
struct CustomVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

// Apply quaternion rotation
fn apply_quaternion_rotation(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qvec = q.xyz;
    let qw = q.w;
    let uv = cross(qvec, v);
    let uuv = cross(qvec, uv);
    return v + 2.0 * (uv * qw + uuv);
}

// Get color based on elevation
fn getColorFromElevation(elevation: f32) -> vec4<f32> {
    if (elevation < -6000.0) {
        return vec4<f32>(8.0 / 255.0, 14.0 / 255.0, 48.0 / 255.0, 1.0);
    } else if (elevation < -3000.0) {
        return vec4<f32>(31.0 / 255.0, 45.0 / 255.0, 71.0 / 255.0, 1.0);
    } else if (elevation < -150.0) {
        return vec4<f32>(42.0 / 255.0, 60.0 / 255.0, 99.0 / 255.0, 1.0);
    } else if (elevation < -50.0) {
        return vec4<f32>(52.0 / 255.0, 75.0 / 255.0, 117.0 / 255.0, 1.0);
    } else if (elevation < 0.0001) {
        return vec4<f32>(87.0 / 255.0, 120.0 / 255.0, 179.0 / 255.0, 1.0);
    } else if (elevation < 75.0) {
        return vec4<f32>(79.0 / 255.0, 166.0 / 255.0, 66.0 / 255.0, 1.0);
    } else if (elevation < 150.0) {
        return vec4<f32>(52.0 / 255.0, 122.0 / 255.0, 42.0 / 255.0, 1.0);
    } else if (elevation < 400.0) {
        return vec4<f32>(0.0 / 255.0, 83.0 / 255.0, 11.0 / 255.0, 1.0);
    } else if (elevation < 1000.0) {
        return vec4<f32>(61.0 / 255.0, 55.0 / 255.0, 4.0 / 255.0, 1.0);
    } else if (elevation < 2000.0) {
        return vec4<f32>(128.0 / 255.0, 84.0 / 255.0, 17.0 / 255.0, 1.0);
    } else if (elevation < 3200.0) {
        return vec4<f32>(151.0 / 255.0, 122.0 / 255.0, 68.0 / 255.0, 1.0);
    } else if (elevation < 5000.0) {
        return vec4<f32>(182.0 / 255.0, 181.0 / 255.0, 181.0 / 255.0, 1.0);
    } else {
        return vec4<f32>(238.0 / 255.0, 238.0 / 255.0, 238.0 / 255.0, 1.0);
    }
}

@vertex
fn vertex(vertex: Vertex) -> CustomVertexOutput {
    var out: CustomVertexOutput;

    let global_index = map_selection.x * map_selection.y + vertex.elevation_index;
    let elevation_meters = f32(elevation_buffer[global_index]);

    let base_sphere_position = vertex.i_pos_scale.xyz;
    let sphere_radius = length(base_sphere_position);
    let sphere_direction = normalize(base_sphere_position);

    let elevation_world_units = elevation_meters * 0.00004;
    let elevated_instance_pos = sphere_direction * (sphere_radius + elevation_world_units);

    let rotated_position = apply_quaternion_rotation(vertex.i_rotation, vertex.position);
    let scaled_position = rotated_position * vertex.i_pos_scale.w;

    let final_position = scaled_position + elevated_instance_pos;
    
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0u),
        vec4<f32>(final_position, 1.0)
    );
    
    let rotated_normal = apply_quaternion_rotation(vertex.i_rotation, vertex.normal);
    
    let world_from_local = get_world_from_local(0u);
    out.world_normal = normalize((world_from_local * vec4<f32>(rotated_normal, 0.0)).xyz);
    
    out.world_position = (world_from_local * vec4<f32>(final_position, 1.0)).xyz;

    let color = getColorFromElevation(elevation_meters);
    out.color = color;
    
    return out;
}

@fragment
fn fragment(in: CustomVertexOutput) -> @location(0) vec4<f32> {
    // Get camera view direction for specular calculations
    let view_dir = normalize(view.world_position.xyz - in.world_position);
    
    // Sun direction - you can adjust this to match your sun's orbit
    // Currently set to give nice lighting from a diagonal angle
    let sun_dir = normalize(vec3<f32>(0.5, 0.7, 0.3));
    
    // Ambient light - base illumination
    let ambient_strength = 0.0;
    let ambient = in.color.rgb * ambient_strength;
    
    // Diffuse lighting - main directional light
    let n_dot_l = max(dot(in.world_normal, sun_dir), 0.0);
    let diffuse = n_dot_l * 0.5 * in.color.rgb;
    
    // Specular highlights for water and snow
    let specular_strength = 0.2;
    let reflect_dir = reflect(-sun_dir, in.world_normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 8.0);
    
    var specular = vec3<f32>(0.0);
    
    // Detect water (bluish) and snow (white) for specular
    let is_water = in.color.b > 0.6 && in.color.r < 0.4;
    let is_snow = in.color.r > 0.7 && in.color.g > 0.7 && in.color.b > 0.7;
    
    if (is_water || is_snow) {
        specular = spec * specular_strength * vec3<f32>(1.0, 1.0, 0.95);
    }
    
    // Combine lighting components
    var final_color = ambient + diffuse + specular;
    
    // Atmospheric perspective (fog effect)
    let distance_factor = length(in.world_position - view.world_position.xyz);
    let fog_density = 0.0000015;
    let fog_factor = exp(-distance_factor * fog_density);
    let fog_color = vec3<f32>(0.75, 0.8, 0.9);
    final_color = mix(fog_color, final_color, fog_factor);
    
    // Tone mapping to prevent overexposure
    final_color = final_color / (final_color + vec3<f32>(1.0));
    
    // Gamma correction for proper display
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(final_color, in.color.a);
}

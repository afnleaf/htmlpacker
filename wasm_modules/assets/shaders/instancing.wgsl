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
    
    let rotated_position = 
        apply_quaternion_rotation(vertex.i_rotation, vertex.position);
    
    let scaled_position = rotated_position * vertex.i_pos_scale.w;

    let instance_position = scaled_position + vertex.i_pos_scale.xyz;
    
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0u),
        vec4<f32>(instance_position, 1.0)
    );
    
    let rotated_normal = 
        apply_quaternion_rotation(vertex.i_rotation, vertex.normal);
    
    let world_from_local = get_world_from_local(0u);
    out.world_normal = normalize(
        (world_from_local * vec4<f32>(rotated_normal, 0.0)).xyz);
    
    out.world_position = (world_from_local * vec4<f32>(instance_position, 1.0)).xyz;
    
    out.color = vertex.i_color;
    
    return out;
}

// this fragment shader runs for every pixel of every prism
// receives interpolated color and normal
// simple directional lighting, calculate diffuse lighting
// calculates simple lighting and returns final "lit" pixel color
@fragment
fn fragment(in: CustomVertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
    let ambient = 0.3;
    
    let n_dot_l = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = n_dot_l * 0.7;
    
    let lit_color = in.color.rgb * (ambient + diffuse);
    
    return vec4<f32>(lit_color, in.color.a);
}


use bevy::{
    //diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    render::{
        mesh::*,
        //render_asset::RenderAssetUsages,
        //render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    image::{Image},
    //scene::{SceneRoot},
    prelude::*,
};

const ELEVATION_DATA_BYTES: &[u8] = include_bytes!("../assets/test.bin.br");

use crate::scene;

#[derive(Resource)]
struct ElevationData {
    elevation: Vec<i16>,
    height: usize, // latitude
    width: usize, // longitude
}

fn decompress_elevation() -> Vec<u8> {
    let mut decompressor = 
        brotli::Decompressor::new(
            std::io::Cursor::new(ELEVATION_DATA_BYTES), 4096);
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(&mut decompressor, &mut decompressed)
        .expect("Failed to decompress data");
    decompressed
}

fn parse_elevation() -> ElevationData {
    let decompressed = decompress_elevation();
    let mut elevation = Vec::with_capacity(decompressed.len() / 2);
    for c in decompressed.chunks_exact(2) {
        elevation.push(
            i16::from_le_bytes([c[0], c[1]]));
    }
    let e = ElevationData {
        elevation,
        height: 181,
        width: 361,
    };
    e
}

fn calculate_vertices(e: &ElevationData) -> Vec<Vec3> {
    let mut vertices: Vec<Vec3> = Vec::with_capacity(e.height * e.width);
    let r = 2.0; // sphere radius
    
    for i in 0..e.height {
        // Map i from [0, height-1] to [-90, 90] degrees (latitude)
        let lat_deg = -90.0 + (i as f64 * 180.0 / (e.height as f64 - 1.0));
        let lat_rad = lat_deg * std::f64::consts::PI / 180.0;
        
        for j in 0..e.width {
            // Map j from [0, width-1] to [-180, 180] degrees (longitude)
            let lon_deg = -180.0 + (j as f64 * 360.0 / (e.width as f64 - 1.0));
            let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
           

            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;
            
            // swap z and y cause bevy has y as up/down
            vertices.push(Vec3::new(x, y, z));
        }
    }
    
    vertices
}
/*
pub fn prism_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
*/

pub fn prism_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    //commands.insert_resource(&e);
    //commands.apply(&mut world);
    let e = parse_elevation();
    let vertices = calculate_vertices(&e);

    // Pre-generate all materials based on elevation ranges
    let elevation_materials = [
        // Define the elevation ranges and corresponding colors
        (-13000.0, -6000.0, [8, 14, 48]),       // 0x080e30
        (-6000.0, -3000.0, [31, 45, 71]),       // 0x1f2d47
        (-3000.0, -150.0, [42, 60, 99]),        // 0x2a3c63
        (-150.0, -50.0, [52, 75, 117]),         // 0x344b75
        (-50.0, 0.0001, [87, 120, 179]),           // 0x5778b3
        (0.0001, 75.0, [79, 166, 66]),             // 0x4fa642
        (75.0, 150.0, [52, 122, 42]),           // 0x347a2a
        (150.0, 400.0, [0, 83, 11]),            // 0x00530b
        (400.0, 1000.0, [61, 55, 4]),           // 0x3d3704
        (1000.0, 2000.0, [128, 84, 17]),        // 0x805411
        (2000.0, 3200.0, [151, 122, 68]),       // 0x977944
        (3200.0, 5000.0, [182, 181, 181]),      // 0xb6b5b5
        (5000.0, f32::MAX, [238, 238, 238])     // 0xeeeeee
    ].iter().map(|(min_e, max_e, color)| {
        (
            *min_e, 
            *max_e, 
            materials.add(StandardMaterial {
                base_color: Color::srgb_u8(color[0], color[1], color[2]),
                ..default()
            })
        )
    }).collect::<Vec<_>>();
    
    // Default material for any elevation that doesn't match our ranges
    /*
    let default_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // red
        ..default()
    });
    */
    
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(scene::uv_debug_texture())),
        ..default()
    });


    let prism_mesh = meshes.add(Cuboid::new(0.05, 0.05, 0.2));
    // In your setup function, change to:
    //let instances = setup_earth_elevation_points(&vertices, &e.elevation, e.height, e.width);
    // actual_range: Ok(Doubles([-9000.0, 6000.0]))
    let max = 6000.0;
    let min = -9000.0;
    let e_scale_f = 0.3;

    let lat_step = 1;
    let lon_step = 1;
    for i in (0..e.height).step_by(lat_step) {
        for j in (0..e.width).step_by(lon_step) {
            let n = i * e.width + j;
            let vertex_pos = vertices[n];
            
            // Skip if data is missing or is water 
            // (approximated as very low elevation)     
            /*
            if n >= e.elevation.len() || e.elevation[n] < -100 {
                continue;
            }
            */
            
            let ev: f32 = e.elevation[n] as f32;
            //let elevation_scale = (ev / 8000.0).max(1.0) * 0.5;
            let es = (ev - min) / (max - min);

            let elevation_scale = 1.0 + (es * e_scale_f);
            let scaled_position = vertex_pos * elevation_scale;

            let material = elevation_materials
                .iter()
                .find(|(min_e, max_e, _)| ev >= *min_e && ev < *max_e)
                .map(|(_, _, material)| material.clone())
                .unwrap_or_else(|| {
                    println!("<{}>", ev); // Log unexpected elevations
                    debug_material.clone()
                });
            
            // Calculate direction and rotation
            let direction = vertex_pos.normalize();
            let rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            
            // Spawn entity with transform
            commands.spawn((
                Mesh3d(prism_mesh.clone()),
                MeshMaterial3d(material.clone()),
                //Transform::from_translation(vertex_pos)
                Transform::from_translation(scaled_position)
                    .with_rotation(rotation)
                    //.with_scale(Vec3::new(es, es, es)),
            ));
        }
    }

}




    /*
    so what we do next, assets, what do we need them for?
    textures -> materials -> meshesh
    models -> objects, characters

    we have a bunch of textures we want to apply
    we want to load them all at the same time in an array of 109 size
let textures: Vec<Handle<Image>> = (1..=109)
        .map(|i| asset_server.load(format!("textures/texture_{}.png", i)))
        .collect();
let material = materials.add(StandardMaterial {
            base_color_texture: Some(textures[i].clone()),
            ..default()
        });
    let mesh_handle = meshes.add(Sphere::default().mesh().uv(32, 18));
    
commands.spawn((
            PbrBundle {
                mesh: mesh_handle.clone(),
                material,
                transform: Transform::from_translation(position),
                ..default()
            },
            Shape, // Your marker component
        ));
    */
    //decompressor.read_to_end(&mut decompressed)
    //println!("{:?}", decompressed);

// bevy vec struct?
/*
fn calculate_vertices(e: &ElevationData) -> Vec<Vec3> {
    let mut vertices: Vec<Vec3> = Vec::with_capacity(e.height * e.width);

    for i in 0..e.height { //latitude
        for j in 0..e.width { //longitude
            //println!("Elevation at {},{}: {}", 
            //    i, j, e.elevation[i * e.width + j]);

            // calculate 3d position from latitude and longitude
            use std::f64::consts::PI;
            //let a = x.tan();
            //let b = x.sin() / x.cos();
            //let r = 6378 // radius in km
            let r = 2.0;
            let a = PI / 180.0;
            let r_la = i as f64 * a;
            let r_lo = j as f64 * a;

            let x = (r * r_la.cos() * r_lo.cos()) as f32;
            let y = (r * r_la.cos() * r_lo.sin()) as f32;
            let z = (r * r_la.sin()) as f32;

            vertices.push(Vec3::new(x, y, z));
            //println!("x{} y{} z{}", x, y, z);
        }
    }

    vertices
}
*/






/*
fn spawn_beam_to_origin(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    target_position: Vec3
) {
    // Calculate the length of the beam
    let length = target_position.length();
    
    if length > 0.0 {
        // Create a 1x1 rectangular extrusion
        let mesh_handle = meshes.add(Extrusion::new(Rectangle::new(0.1, 0.1), length));
        
        // Calculate midpoint for positioning
        let midpoint = target_position / 2.0;
        
        // Calculate rotation to align with the target direction
        // Default extrusion is along the z-axis, so rotate from z to target
        let direction = target_position.normalize();
        let rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        
        // Spawn the entity
        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(midpoint)
                   .with_rotation(rotation),
            //Shape,
        ));
    }
}
*/

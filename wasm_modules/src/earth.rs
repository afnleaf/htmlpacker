use bevy::{
    //diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    render::{
        mesh::*,
        render_resource::{
            PrimitiveTopology,
            Extent3d,
            TextureDimension,
            TextureFormat,
            Face,
        },
        render_asset::RenderAssetUsages,
        //render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    image::{Image},
    //scene::{SceneRoot},
    prelude::*,
};
use std::f32::consts::{FRAC_PI_2, PI };
use std::path::PathBuf;
use bevy_embedded_assets::EmbeddedAssetReader;
//use crate::scene;
use crate::CurrentMap;


struct RawData {
    data: &'static [u8],
    height: usize,
    width: usize,
}

/*
const ELEVATION_DATA_L: RawData = RawData {
    data: include_bytes!("../assets/test3.bin.br"),
    height: 1801,
    width: 3601,
};
*/

const ELEVATION_DATA_S: RawData = RawData {
    data: include_bytes!("../assets/deg1/1.br"),
    height: 181,
    width: 361
};


//const ELEVATION_DATA_BYTES: &[u8] = include_bytes!("../assets/test3.bin.br");
//const ELEVATION_DATA_SMALL: &[u8] = include_bytes!("../assets/test.bin.br");

#[derive(Resource)]
pub struct AllMapData {
    maps: Vec<ElevationData>,
}

pub fn load_map_data(mut commands: Commands) {
    let maps = load_and_parse_maps_deg1();
    commands.insert_resource(AllMapData { maps });
}

#[derive(Component)]
pub struct MapEntity;


#[derive(Resource)]
pub struct ElevationData {
    elevation: Vec<i16>,
    height: usize, // latitude
    width: usize, // longitude
}

fn decompress_elevation(data: &[u8]) -> Vec<u8> {
    let mut decompressor = 
        brotli::Decompressor::new(
            std::io::Cursor::new(data), 4096);
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(&mut decompressor, &mut decompressed)
        .expect("Failed to decompress data");
    decompressed
}

// s means
fn parse_elevation(data: RawData) -> ElevationData {
    let decompressed = decompress_elevation(data.data);
    let mut elevation = Vec::with_capacity(decompressed.len() / 2);
    for c in decompressed.chunks_exact(2) {
        elevation.push(
            i16::from_le_bytes([c[0], c[1]]));
    }

    let e = ElevationData {
        elevation,
        height: data.height,
        width: data.width,
    };
    e
}

// goes from -90 to +90 and -180 to +180
fn calculate_vertices_small(e: &ElevationData) -> Vec<Vec3> {
    let mut vertices: Vec<Vec3> = Vec::with_capacity(e.height * e.width);
    //let r = 2.0_f64; // sphere radius
    let r = 6.378_f64;
    
    //for i in (0..e.height).rev() {
    for i in 0..e.height {
        // Map i from [0, height-1] to [-90, 90] degrees (latitude)
        let lat_deg = -90.0 + (i as f64 * 180.0 / (e.height as f64 - 1.0));
        let lat_rad = lat_deg * std::f64::consts::PI / 180.0;
        
        //for j in (0..e.width).rev() {
        for j in 0..e.width {
            // Map j from [0, width-1] to [-180, 180] degrees (longitude)
            let lon_deg = -180.0 + (j as f64 * 360.0 / (e.width as f64 - 1.0));
            let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
           
            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;

            vertices.push(Vec3::new(x, y, z));
        }
    }
    
    vertices
}

// goes from +90 to -90 and +180 to -180
fn calculate_vertices_large(e: &ElevationData) -> Vec<Vec3> {
    let mut vertices: Vec<Vec3> = Vec::with_capacity(e.height * e.width);
    //let r = 2.0_f64; // sphere radius
    let r = 6.378_f64;

    // Iterate through latitude (i) and longitude (j) indices
    for i in 0..e.height {
        // Map i from [0, height-1] to [90, -90] degrees (latitude)
        // Starting at North Pole (90°) and going down to South Pole (-90°)
        let lat_deg = 90.0 - (i as f64 * 180.0 / (e.height as f64 - 1.0));
        let lat_rad = lat_deg * std::f64::consts::PI / 180.0;
        
        for j in 0..e.width {
            // Map j from [0, width-1] to [-180, 180] degrees (longitude)
            // Starting at International Date Line (-180°) and going eastward
            let lon_deg = -180.0 + (j as f64 * 360.0 / (e.width as f64 - 1.0));
            let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
           
            // Standard spherical to Cartesian coordinate conversion
            // For consistent orientation with 3D graphics conventions:
            // In typical 3D space, +Y is up, but for Earth, +Z is often North
            // So we map: longitude to XY-plane, latitude to Z
            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;
            
            vertices.push(Vec3::new(x, y, z));
        }
    }
    
    vertices
}

pub fn load_and_parse_maps_deg1() -> Vec<ElevationData> {
    let embedded = EmbeddedAssetReader::preloaded();
    let mut map_data: Vec<ElevationData> = Vec::with_capacity(109);
    for i in 1..=109 {
        let file_path = format!("deg1/{}.br", i);
        match embedded.load_path_sync(&PathBuf::from(&file_path)) {
            Ok(reader) => {
                let r = RawData {
                    data: reader.0,
                    height: 181,
                    width: 361,
                };
                let e = parse_elevation(r);
                map_data.push(e);
            },
            Err(err) => {
                println!("Failed to load file {}: {:?}", i, err);
            }
        }
    }

    map_data
}

pub fn update_earth(
    mut commands: Commands,
    current_map: Res<CurrentMap>,
    all_map_data: Res<AllMapData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_entities: Query<Entity, With<MapEntity>>,
) {
    // take not of the use of is_changed()
    // we only run this function when it is true
    if !current_map.is_changed() {
        return;
    }

    // remove all prisms from the scene
    for entity in map_entities.iter() {
        commands.entity(entity).despawn();
    }

    let e = &all_map_data.maps[current_map.index];

    render_earth(
        &mut commands,
        e,
        &mut meshes,
        &mut images,
        &mut materials,
    );
}

pub fn render_earth(
    commands: &mut Commands,
    e: &ElevationData,
    meshes: &mut ResMut<Assets<Mesh>>,
    images: &mut ResMut<Assets<Image>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let vertices = calculate_vertices_small(e);

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
                perceptual_roughness: 1.0,
                cull_mode: Some(Face::Back),
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
        //base_color_texture: Some(images.add(scene::uv_debug_texture())),
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });


    let prism_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.4));
    
    // In your setup function, change to:
    //let instances = setup_earth_elevation_points(&vertices, &e.elevation, e.height, e.width);
    // actual_range: Ok(Doubles([-9000.0, 6000.0]))
    //let max = 6000.0;
    //let min = -9000.0;
    let max = 8848.86; // mt everest
    let min = -10909.0; // marianas trench
    let e_scale_f = 0.2;

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
            /*
            let direction = vertex_pos.normalize();
            let direction_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            let x_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let rotation = direction_rotation * x_rotation;
            */
            // Create rotation quaternion for π/2 around x-axis
            let rotation_x = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let rotation_y = Quat::from_rotation_y(std::f32::consts::PI);
            let combined_rotation = rotation_x * rotation_y;
            
            // Apply rotation to the scaled_position vector itself
            let rotated_position = combined_rotation.mul_vec3(scaled_position);
            
            // Then calculate direction using the rotated position
            //let direction = rotated_position.normalize();
            let direction = if rotated_position.length() > f32::EPSILON {
                rotated_position.normalize()
            } else {
                Vec3::Y
            };
            let orientation = Quat::from_rotation_arc(Vec3::Z, direction);            
            // Spawn entity with transform
            commands.spawn((
                Mesh3d(prism_mesh.clone()),
                MeshMaterial3d(material.clone()),
                //Transform::from_translation(vertex_pos)
                Transform::from_translation(rotated_position)
                    .with_rotation(orientation),
                    //.with_scale(Vec3::new(es, es, es)),
                MapEntity,
            ));
        }
    }

}

pub fn prism_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    //mut current_map: ResMut<CurrentMap>,
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    all_map_data: Res<AllMapData>,
) {
    render_earth(
        &mut commands,
        &all_map_data.maps[current_map.index],
        &mut meshes,
        &mut images,
        &mut materials
    );


    /*
    let embedded = EmbeddedAssetReader::preloaded();
    let i = 32;
    let file_path = format!("deg1/{}.br", i);
    let reader = embedded.load_path_sync(&PathBuf::from(&file_path)).unwrap();
    let r = RawData {
        data: reader.0,
        height: 181,
        width:  361,
    };
    let e = parse_elevation(r);
    */
    //commands.insert_resource(&e);
    //commands.apply(&mut world);
    //let e = parse_elevation(ELEVATION_DATA_S);
    //let vertices = calculate_vertices_small(&e);
    /*
    let map_data = load_and_parse_maps_deg1();
    let e = &map_data[current_map.index];
    let vertices = calculate_vertices_small(&e);


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
                perceptual_roughness: 1.0,
                cull_mode: Some(Face::Back),
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
        //base_color_texture: Some(images.add(scene::uv_debug_texture())),
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });


    let prism_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.4));
    
    // In your setup function, change to:
    //let instances = setup_earth_elevation_points(&vertices, &e.elevation, e.height, e.width);
    // actual_range: Ok(Doubles([-9000.0, 6000.0]))
    //let max = 6000.0;
    //let min = -9000.0;
    let max = 8848.86; // mt everest
    let min = -10909.0; // marianas trench
    let e_scale_f = 0.2;

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
            /*
            let direction = vertex_pos.normalize();
            let direction_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            let x_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let rotation = direction_rotation * x_rotation;
            */
            // Create rotation quaternion for π/2 around x-axis
            let rotation_x = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let rotation_y = Quat::from_rotation_y(std::f32::consts::PI);
            let combined_rotation = rotation_x * rotation_y;
            
            // Apply rotation to the scaled_position vector itself
            let rotated_position = combined_rotation.mul_vec3(scaled_position);
            
            // Then calculate direction using the rotated position
            //let direction = rotated_position.normalize();
            let direction = if rotated_position.length() > f32::EPSILON {
                rotated_position.normalize()
            } else {
                Vec3::Y
            };
            let orientation = Quat::from_rotation_arc(Vec3::Z, direction);            
            // Spawn entity with transform
            commands.spawn((
                Mesh3d(prism_mesh.clone()),
                MeshMaterial3d(material.clone()),
                //Transform::from_translation(vertex_pos)
                Transform::from_translation(rotated_position)
                    .with_rotation(orientation),
                    //.with_scale(Vec3::new(es, es, es)),
                MapEntity,
            ));
        }
    }
    */
}


// its the same performance?
pub fn prism_earth2(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    //commands.insert_resource(&e);
    //commands.apply(&mut world);
    let e = parse_elevation(ELEVATION_DATA_S);
    let vertices = calculate_vertices_small(&e);

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
                perceptual_roughness: 1.0,
                cull_mode: Some(Face::Back),
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
        //base_color_texture: Some(images.add(scene::uv_debug_texture())),
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });


    let prism_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.4));
    
    // In your setup function, change to:
    //let instances = setup_earth_elevation_points(&vertices, &e.elevation, e.height, e.width);
    // actual_range: Ok(Doubles([-9000.0, 6000.0]))
    //let max = 6000.0;
    //let min = -9000.0;
    let max = 8848.86; // mt everest
    let min = -10909.0; // marianas trench
    let e_scale_f = 0.2;

    // Group transforms by material
    let mut instance_groups: Vec<(Handle<StandardMaterial>, Vec<Transform>)> = 
        elevation_materials.iter().map(|(_, _, mat)| (mat.clone(), Vec::new())).collect();

    // Assign each prism to a material group
    for i in 0..e.height {
        for j in 0..e.width {
            /* 
            let n = i * e.width + j;
            let vertex_pos = vertices[n];
            let ev = e.elevation[n] as f32;

            // Calculate transform (as in your original code)
            let transform = Transform::from_xyz(
                vertex_pos[0] as f32 - 180.0,
                vertex_pos[1] as f32,
                vertex_pos[2] as f32 - 90.0,
            );
            */            
            
            let n = i * e.width + j;
            let vertex_pos = vertices[n];
            let ev: f32 = e.elevation[n] as f32;
            let es = (ev - min) / (max - min);
            let elevation_scale = 1.0 + (es * e_scale_f);
            let scaled_position = vertex_pos * elevation_scale;
            // Create rotation quaternion for π/2 around x-axis
            let rotation_x = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let rotation_y = Quat::from_rotation_y(std::f32::consts::PI);
            let combined_rotation = rotation_x * rotation_y;
            // Apply rotation to the scaled_position vector itself
            let rotated_position = combined_rotation.mul_vec3(scaled_position);
            // Then calculate direction using the rotated position
            //let direction = rotated_position.normalize();
            let direction = if rotated_position.length() > f32::EPSILON {
                rotated_position.normalize()
            } else {
                Vec3::Y
            };
            let orientation = Quat::from_rotation_arc(Vec3::Z, direction);            
            let transform = Transform::from_translation(rotated_position)
                    .with_rotation(orientation);

            // Find the material group
            let mat_idx = elevation_materials
                .iter()
                //.position(|(min_e, max_e, _)| elevation >= *min_e && elevation < *max_e)
                .position(|(min_e, max_e, _)| ev >= *min_e && ev < *max_e)
                .unwrap_or(0);
            instance_groups[mat_idx].1.push(transform);
        }
    }

    // Spawn entities, encouraging batching
    for (material, transforms) in instance_groups {
        if !transforms.is_empty() {
            for transform in transforms {
                //commands.spawn(PbrBundle {
                //    mesh: prism_mesh.clone(),
                //    material: material.clone(),
                //    transform,
                //    ..default()
                //});
                commands.spawn((
                    Mesh3d(prism_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    transform
                ));

            }
        }
    }
}




/// Creates a colorful test pattern
pub fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}






// archive


            // Calculate normals with strict normalization
            /*
            for k in 0..4 {
                // Calculate normal based on the local terrain slope
                let v = scaled_vertices[k];
                
                // Find neighboring vertices
                let dx = if k % 2 == 0 { 
                    scaled_vertices[k+1] - v 
                } else { 
                    v - scaled_vertices[k-1] 
                };
                
                let dy = if k < 2 { 
                    scaled_vertices[k+2] - v 
                } else { 
                    v - scaled_vertices[k-2] 
                };
                
                // Cross product for normal with extra normalization step
                let mut normal = dx.cross(dy);
                
                // Ensure perfect normalization by dividing by exact length
                let length = normal.length();
                if length > 1e-6 {  // Prevent division by zero
                    normal = normal / length;
                } else {
                    normal = Vec3::Y; // Default fallback normal
                }
                
                // Verify length is exactly 1.0 (or extremely close)
                debug_assert!((normal.length() - 1.0).abs() < 1e-6);
                
                mesh_normals.push(normal);
            }
            */
    /*
    // Create material with better lighting properties
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.8, 0.8, 0.8),
        // Lower roughness for a more reflective surface
        perceptual_roughness: 1.0,
        // Add some metallic property for better light reflection
        //metallic: 0.1,
        // Increase reflectance for better lighting
        //reflectance: 0.3,
        ..default()
    });
     /*
    // Helper function to get color based on elevation
    fn get_elevation_color(elevation: f32) -> [f32; 4] {
        // Create the color values with reduced brightness/saturation
        let (r, g, b) = match elevation {
            e if e < -6000.0 => (6, 10, 34),     // Darker deep ocean
            e if e < -3000.0 => (21, 32, 51),    // Darker ocean
            e if e < -150.0 => (30, 43, 71),     // Darker shallow ocean
            e if e < -50.0 => (37, 53, 83),      // Darker coastal water
            e if e < 0.0001 => (62, 86, 128),    // Darker shoreline
            e if e < 75.0 => (56, 118, 47),      // Darker low land
            e if e < 150.0 => (37, 87, 30),      // Darker mid land
            e if e < 400.0 => (0, 59, 8),        // Darker forest/vegetation
            e if e < 1000.0 => (43, 39, 3),      // Darker low mountains
            e if e < 2000.0 => (91, 60, 12),     // Darker mountains
            e if e < 3200.0 => (107, 86, 48),    // Darker high mountains
            e if e < 5000.0 => (130, 129, 129),  // Darker snow line
            _ => (170, 170, 170),                // Darker peaks
        };
        
        // Convert from 0-255 range to 0.0-1.0 range directly
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
    }
    */   */// Calculate UV coordinates based on normalized position on the sphere
            /*
            for k in 0..4 {
                // Calculate UV coordinates from position on the sphere
                // This is a simple equirectangular projection
                let pos = scaled_vertices[k].normalize();
                let u = (PI + pos.z.atan2(pos.x)) / (2.0 * PI);
                let v = (FRAC_PI_2 - pos.y.asin()) / PI;
                
                mesh_uvs.push([u, v]);
            }
            */


// MESH WAY OF DOIGN THIS

// Add this helper function to create a half-intensity white texture
fn create_half_intensity_texture(textures: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    // Create a single white pixel texture with reduced intensity
    let mut texture_data = [0; 4]; // RGBA
    texture_data[0] = 180; // R - reduced from 255
    texture_data[1] = 180; // G - reduced from 255
    texture_data[2] = 180; // B - reduced from 255
    texture_data[3] = 255; // A - full alpha
    
    // Create a 1x1 texture
    let texture = Image::new(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        texture_data.to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    
    textures.add(texture)
}

fn fract(x: f32) -> f32 {
    x - x.floor()
}

// Helper function to get color based on elevation
fn get_elevation_color(elevation: f32) -> [f32; 4] {
    // Create the color values directly without depending on Color methods
    let (r, g, b) = match elevation {
        e if e < -6000.0 => (8, 14, 48),     // 0x080e30
        e if e < -3000.0 => (31, 45, 71),    // 0x1f2d47
        e if e < -150.0 => (42, 60, 99),     // 0x2a3c63
        e if e < -50.0 => (52, 75, 117),     // 0x344b75
        e if e < 0.0001 => (87, 120, 179),   // 0x5778b3
        e if e < 75.0 => (79, 166, 66),      // 0x4fa642
        e if e < 150.0 => (52, 122, 42),     // 0x347a2a
        e if e < 400.0 => (0, 83, 11),       // 0x00530b
        e if e < 1000.0 => (61, 55, 4),      // 0x3d3704
        e if e < 2000.0 => (128, 84, 17),    // 0x805411
        e if e < 3200.0 => (151, 122, 68),   // 0x977944
        e if e < 5000.0 => (182, 181, 181),  // 0xb6b5b5
        _ => (238, 238, 238),                // 0xeeeeee
    };
    
    // Convert from 0-255 range to 0.0-1.0 range directly
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

pub fn earth_terrain_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>
) {
    let e = parse_elevation(ELEVATION_DATA_S);
    let vertices = calculate_vertices_large(&e);
    
    let max = 8848.86; // mt everest
    let min = -10909.0; // marianas trench
    let e_scale_f = 0.2;
    //let e_scale_f = 0.06;
    //let e_scale_f = 0.00309781436186892442772028849169;
    
    // Create a new mesh from scratch - now with RenderAssetUsages parameter
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    
    // Calculate vertices and colors for the mesh
    let mut mesh_vertices = Vec::new();
    let mut mesh_colors = Vec::new();
    let mut mesh_uvs = Vec::new();
    let mut mesh_normals = Vec::new();
    let mut indices = Vec::new();
    
    let lat_step = 1;
    let lon_step = 1;
    
    // Generate vertices with elevation offset
    //for i in (0..e.height - lat_step).step_by(lat_step) {
    //    for j in (0..e.width - lon_step).step_by(lon_step) {
    for i in (0..e.height).step_by(lat_step) {
        for j in (0..e.width).step_by(lon_step) {
            // Get the four corners of a quad
            let idx = [
                i * e.width + j,
                i * e.width + (j + lon_step),
                (i + lat_step) * e.width + j,
                (i + lat_step) * e.width + (j + lon_step),
            ];
            
            // Skip if any index is out of bounds
            if idx.iter().any(|&n| n >= e.elevation.len()) {
                continue;
            }
            
            // Get elevation data and scale vertices
            let mut scaled_vertices = [Vec3::ZERO; 4];
            let mut colors = [[0.0, 0.0, 0.0, 1.0]; 4]; // RGBA float arrays

            // Elevation scale and color
            for k in 0..4 {
                let ev: f32 = e.elevation[idx[k]] as f32;
                let es = (ev - min) / (max - min);
                let elevation_scale = 1.0 + (es * e_scale_f);
                
                // Apply scale and rotation
                let rotation_x = Quat::from_rotation_x(FRAC_PI_2);
                let rotation_y = Quat::from_rotation_y(PI);
                let combined_rotation = rotation_x * rotation_y;
                
                scaled_vertices[k] = combined_rotation
                    .mul_vec3(vertices[idx[k]] * elevation_scale);
                //scaled_vertices[k] = vertices[idx[k]] * elevation_scale;
                //scaled_vertices[k] = rotation_x
                //    .mul_vec3(vertices[idx[k]] * elevation_scale);
                colors[k] = get_elevation_color(ev);
            }
            
            // Add vertices to the mesh
            let base_idx = mesh_vertices.len() as u32;
            mesh_vertices.extend_from_slice(&scaled_vertices);
            mesh_colors.extend_from_slice(&colors);
           

            
            // UV coordinates from position on the sphere
            for k in 0..4 {
                //let pos = scaled_vertices[k].normalize();
                let pos = if scaled_vertices[k].length() > f32::EPSILON {
                    scaled_vertices[k].normalize()
                } else {
                    Vec3::Y
                };
                
                // Basic equirectangular projection
                let mut u = (PI + pos.z.atan2(pos.x)) / (2.0 * PI);
                let mut v = (FRAC_PI_2 - pos.y.asin()) / PI;
                
                // Flip the texture horizontally (fix mirroring)
                u = 1.0 - u;
                
                // Rotate to align with elevation data
                // Try different values here to find the right alignment
                // Start with 0.25 and adjust as needed
                u = fract(u + 0.50);
                
                // Optional: Flip vertically if needed
                // v = 1.0 - v;
                
                mesh_uvs.push([u, v]);
            }

            // Normals
            // For spherical terrain, use position-based normals
            for k in 0..4 {
                // This works better for global shape illumination
                //let base_normal = scaled_vertices[k].normalize();
                let base_normal = if scaled_vertices[k].length() > f32::EPSILON {
                    scaled_vertices[k].normalize()
                } else {
                    Vec3::Y
                };
                
                // Blend with terrain-based normal for detail
                let v = scaled_vertices[k];
                let dx = if k % 2 == 0 { scaled_vertices[k+1] - v } else { v - scaled_vertices[k-1] };
                let dy = if k < 2 { scaled_vertices[k+2] - v } else { v - scaled_vertices[k-2] };
                
                // Calculate terrain normal
                let mut terrain_normal = dy.cross(dx); // Note: reversed order for correct orientation
                if terrain_normal.length() > 1e-6 {
                    terrain_normal = terrain_normal.normalize();
                } else {
                    terrain_normal = base_normal;
                }
                
                // Blend between base sphere normal and terrain normal
                // Higher weight for base_normal gives smoother lighting
                let normal = (base_normal * 0.7 + terrain_normal * 0.3).normalize();
                mesh_normals.push(normal);
            }
            
            // Create two triangles for the quad
            indices.extend_from_slice(&[
                base_idx, base_idx + 2, base_idx + 1,
                base_idx + 1, base_idx + 2, base_idx + 3,
            ]);
            // for mini mode
            /*
            indices.extend_from_slice(&[
                base_idx, base_idx + 1, base_idx + 2,
                base_idx + 1, base_idx + 3, base_idx + 2,
            ]);
            */
        }
    }
    
    // Set mesh data
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_colors);
    mesh.insert_indices(Indices::U32(indices));
   
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        // Make it less shiny/reflective
        perceptual_roughness: 1.0,
        // Remove any metallic property
        metallic: 0.0,
        // Lower reflectance for a more matte appearance
        reflectance: 0.0,
        // reduce the base color's intensity to make it less "white-washed"
        //base_color_texture: Some(create_half_intensity_texture(&mut textures)),
        // Enable vertex colors
        alpha_mode: AlphaMode::Opaque,
        // Reduce how much light is reflected
        //diffuse_transmission: 0.0,
        //cull_mode: None,
        cull_mode: Some(Face::Back),
        ..default()
    });
    /*
    let texture_handle = asset_server.load("textures/texture1.png");
    let material = materials.add(StandardMaterial {
        //base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        diffuse_transmission: 0.0,
        ..default()
    });
    */
    // Use the recommended Mesh3d and MeshMaterial3d components
    /*
    let entity = commands.spawn_empty().id();
    commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
    commands.entity(entity).insert(MeshMaterial3d(material));
    */

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        //Transform::from_scale(Vec3::splat(1.0)),
        //GlobalTransform::default(),
    ));
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

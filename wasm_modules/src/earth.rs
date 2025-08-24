/*
earth.rs

prepares data for shader buffer
load elevation data from assets
create static instance geometry
*/


use std::path::PathBuf;

use bevy::{
    prelude::*,
    ecs::{
        query::QueryItem,
    },
    render::{
        extract_resource::ExtractResource,
        extract_component::ExtractComponent,
        view::{
            NoFrustumCulling, 
        },
    },
};
use bevy_embedded_assets::EmbeddedAssetReader;
use bytemuck::{Pod, Zeroable};


// NOT IMPLEMENTED
// to determine resolution of data
// based on lat/lon of earth
// 180 lat (-90 to 90 = 181?)
// 360 lon (-180 to 180 = 361?) 
// at 1 degree
// this can be further decomposed into minutes and seconds
// our larger resolution is 6 mins
enum MapSize {
    Deg1, //    65,341
    Min6, // 6,485,401
}


/*
pre processing step to effiently build earth from a single mesh
take .br -> decompress -> get elevation buffer
pre calc base vertices once, calc elevation index
now instance can read elevation grid based on (181, 361) * n
*/ 

// parse elevation buffers ----------------------------------------------------

// internal representation
//#[derive(Resource, Clone)]
#[derive(Clone, Default)]
pub struct ElevationBuffer {
    pub buffer: Vec<i32>,
    //pub height: usize, // latitude 180
    //pub width: usize, // longitude 360
}

#[derive(Resource, Default, Clone, ExtractResource)]
pub struct AllMapData {
    pub maps: Vec<ElevationBuffer>,
}

// load elevation from assets folder
pub fn load_elevation_buffers(mut commands: Commands) {
    let maps = load_and_parse_maps_deg1();
    println!("loading all map data");
    commands.insert_resource(AllMapData { maps });
}

// returns empty vec on failure
fn decompress_elevation(data: &[u8]) -> Vec<u8> {
    let mut decompressor = 
        brotli::Decompressor::new(
            std::io::Cursor::new(data), 4096);
    let mut decompressed = Vec::new();
    std::io::Read::read_to_end(&mut decompressor, &mut decompressed)
        .expect("Failed to decompress data");
    decompressed
}

// we do this intermediate step to streamline our i16 internal representation
// have to do this due to endianess of the .br data
fn bytes_to_i16_vec(bytes: &[u8]) -> Vec<i32> {
    bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as i32)
        .collect()
}

// parse all the elevation data files here
// we need an asset reader to read the "asset/deg1/{filepath}"
// read all maps in 1 by 1
pub fn load_and_parse_maps_deg1() -> Vec<ElevationBuffer> {
    let embedded = EmbeddedAssetReader::preloaded();
    let mut map_data: Vec<ElevationBuffer> = Vec::with_capacity(109);
    for i in 1..=109 {
        let file_path = format!("deg1/{}.br", i);
        match embedded.load_path_sync(&PathBuf::from(&file_path)) {
            Ok(reader) => {
                // decompress datafile
                // convert to elevation vec of i16s
                let elevation_buffer_raw = decompress_elevation(reader.0);
                let elevation_buffer = bytes_to_i16_vec(&elevation_buffer_raw);
                //println!("i16len: {}", elevation_buffer.len());
                // on len small or len big switch between 1deg and 6min?
                let e = ElevationBuffer {
                    buffer: elevation_buffer,
                    //height: 181,
                    //width: 361,
                };
                map_data.push(e);
            },
            Err(err) => {
                println!("Failed to load file {}: {:?}", i, err);
            }
        }
    }

    map_data
}


// static prism instances -----------------------------------------------------

// the buffer that gets sent to the shader
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    position: Vec3,
    scale: Vec3,
    _padding1: [f32; 2],
    rotation: Quat,
    color: [f32; 4],
    elevation_index: u32,
    _padding2: [f32; 3],
}

// all of the instance properties together in a vec as a component
// deref trait when used for a struct wrap around lets the compiler know
// that if a method or field is used on this struct but doesn't exist
// look at the type that it wraps
#[derive(Component, Deref)]
pub struct InstanceMaterialData(Vec<InstanceData>);

// getting out each material component from the vec of instances
// bridge between main world (game logic) and render world (render logic) 
impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(
        item: QueryItem<'_, Self::QueryData>
    ) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

// our basic setup (bad still)
pub fn setup_instance_geometries(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // configure sphere parameters
    let height = 181;
    let width = 361;
    
    // sphere radius (6378km = earth radius)
    let r = 6.378_f64;
    
    // Generate sphere points and convert to InstanceData
    let instance_data: Vec<InstanceData> = 
    (0..height).flat_map(|i| {
            // Map i from [0, height-1] to [-90, 90] degrees (latitude)
            let lat_deg = -90.0 + (i as f64 * 180.0 / (height as f64 - 1.0));
            let lat_rad = lat_deg * std::f64::consts::PI / 180.0;

            // Calculate latitude-based scale
            // smaller at poles, larger at equator
            // cos(lat) gives us 1.0 at equator, ~0 at poles
            
            //let lat_scale = lat_rad.cos() as f32;  
            //let base_scale = 1.0;
            //let min_scale = 0.5;
            //let scaled = base_scale * lat_scale;
            //let mut scale = scaled.max(min_scale);
            //scale = 1.0;
            // Calculate latitude-based scale for longitude compression
            let lat_scale = lat_rad.cos().abs() as f32;  
            
            // Base scale for 1-degree grid cells
            let base_scale = 0.15;   // Overall size at equator
            let min_lon_scale = 0.75; // Minimum longitude scale at poles (20% of equator)
            
            // Non-uniform scaling: 
            // - X and Z (east-west) scale down with latitude
            // - Y (north-south) remains constant
            let lon_scale = base_scale * (min_lon_scale + (1.0 - min_lon_scale) * lat_scale);
            let lat_scale_uniform = base_scale; // Constant for latitude direction
            
            // Create Vec3 scale
            let scale = Vec3::new(
                lon_scale,           // X scale (varies with latitude)
                lat_scale_uniform,   // Y scale (constant)
                lon_scale,           // Z scale (varies with latitude)
            );

            //let lat_factor = lat_rad.cos().abs().sqrt() as f32;
            //let scale = 0.67 * (0.7 + 0.3 * lat_factor);

            (0..width).map(move |j| {
                // Map j from [0, width-1] to [-180, 180] degrees (longitude)
                let lon_deg = -180.0 + (j as f64 * 360.0 / (width as f64 - 1.0));
                let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
                
                // Cartesian conversion
                let x = (r * lat_rad.cos() * lon_rad.sin()) as f32;
                let y = (r * lat_rad.sin()) as f32;
                let z = (r * lat_rad.cos() * lon_rad.cos()) as f32;

                let position = Vec3::new(x, y, z);
                let normal = position.normalize();
                // Calculate rotation to orient the cube outward from origin
                // This rotates from default Y-up to point along the normal
                let rotation = if normal.y.abs() > 0.999 {
                    // Special case for poles to avoid numerical instability
                    if normal.y > 0.0 {
                        Quat::IDENTITY
                    } else {
                        Quat::from_rotation_x(std::f32::consts::PI)
                    }
                } else {
                    Quat::from_rotation_arc(Vec3::Y, normal)
                };

                let color = LinearRgba::from(Color::hsla(
                    (lon_deg + 180.0) as f32, // Hue based on longitude
                    0.7,                      // Saturation
                    0.5,                      // Lightness
                    1.0                       // Alpha
                )).to_f32_array();

                let elevation_index = (i * width + j) as u32;
                //println!("{}", elevation_index);

                InstanceData {
                    position,
                    scale, 
                    _padding1: [0.0; 2],
                    rotation,
                    color,
                    elevation_index,
                    _padding2: [0.0; 3],
                }
            })
        })
        .collect();

    println!("Creating {} instances", instance_data.len());
    println!("InstanceData size: {} bytes", std::mem::size_of::<InstanceData>());
    
    // Spawn the instanced mesh
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 5.0, 1.0))),
        //Mesh3d(meshes.add(Cylinder::default())),
        InstanceMaterialData(instance_data),
        NoFrustumCulling,
    ));
}


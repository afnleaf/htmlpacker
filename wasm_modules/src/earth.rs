//use std::f32::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::image::Image;
use bevy::render::{
    //mesh::*,
    render_resource::{
        //VertexAttribute, VertexFormat, VertexBufferLayout, VertexStepMode,
        Extent3d, TextureDimension, TextureFormat, 
        //Face,
    },
    render_asset::RenderAssetUsages,
};
//use bevy::pbr::InstanceBuffer;

use bevy_embedded_assets::EmbeddedAssetReader;
//use bytemuck::{Pod, Zeroable};
//use crate::CurrentMap;

/*
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
    prelude::*,
};

use bevy::pbr::InstanceBuffer;
use bytemuck::{Pod, Zeroable};

use std::f32::consts::{FRAC_PI_2, PI };
use std::path::PathBuf;
use bevy_embedded_assets::EmbeddedAssetReader;

use crate::CurrentMap;
*/

/* pre calculation step ---------------------------------------------------  */

/*
take .br -> decompress -> read grid -> calc base vertices
no
we pre calc base vertices once
decompress -> i16 read grid based on 181, 361
one inner outer for loop for less repetition
*/ 


// internal representation
#[derive(Resource)]
pub struct ElevationBuffer {
    pub buffer: Vec<i16>,
    pub height: usize, // latitude 180
    pub width: usize, // longitude 360
}

#[derive(Component)]
pub struct MapEntity;

#[derive(Resource)]
pub struct AllMapData {
    pub maps: Vec<ElevationBuffer>,
}

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

// parse elevation buffers

// load elevation from assets folder
pub fn load_elevation_buffers(mut commands: Commands) {
    let maps = load_and_parse_maps_deg1();
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
fn bytes_to_i16_vec(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect()
}

/*
* parse all the elevation data files here
* we need an asset reader to read the "asset/deg1/{filepath}"
* read all maps in 1 by 1
*/
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
                    height: 181,
                    width: 361,
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

// create our vertices

// internal representation
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticGeometry {
    position: [f32; 3],
    //_pad1: f32,
    rotation: [f32; 4],
    normal: [f32; 3],
    //_pad2: f32,
}

#[derive(Resource)]
pub struct AllGeometry {
    pub instances: Vec<StaticGeometry>,
}

pub fn generate_static_geometry(mut commands: Commands) {
    commands.insert_resource(AllGeometry { 
        instances: create_all_static_geometry() 
    });
} 

fn create_all_static_geometry() -> Vec<StaticGeometry> {
    // figure out how to do this based on data size
    let height = 181;
    let width = 361;
    let n = height * width;
    let mut vec_static_geometry: Vec<StaticGeometry> = Vec::with_capacity(n);
    // sphere radius, real earth size is 6378km radius
    let r = 6.378_f64;

    // loop through each point
    for i in 0..height {
        // map i from [0, height-1] to [-90, 90] degrees (latitude)
        let lat_deg = -90.0 + (i as f64 * 180.0 / (height as f64 - 1.0));
        let lat_rad = lat_deg * std::f64::consts::PI / 180.0;

        //for j in (0..e.width).rev() {
        for j in 0..width {
            // map j from [0, width-1] to [-180, 180] degrees (longitude)
            let lon_deg = -180.0 + (j as f64 * 360.0 / (width as f64 - 1.0));
            let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
            // cartesian conversion 
            let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
            let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
            let z = (r * lat_rad.sin()) as f32;

            let position = Vec3::new(x, y, z);
            let normal = position.normalize();
            let rotation = Quat::from_rotation_arc(Vec3::Y, normal);

            let g = StaticGeometry {
                position: position.into(),
                normal: normal.into(),
                rotation: rotation.into(),
            };

            vec_static_geometry.push(g);
        }
    }

    vec_static_geometry
}

/*
pub fn spawn_static_geometry(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    all_geometry: Res<AllGeometry>,
) {
    // our single rect prism
    let prism_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.4));
    // texture
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let instance_buffer = InstanceBuffer::from(
        all_geometry.instances.as_slice()
    );

    commands.spawn((
        Mesh3d(prism_mesh),
        MeshMaterial3d(debug_material),
        Transform::default(),
        instance_buffer, // single GPU buffer with all instances
    ));
}
*/


// Creates a colorful test pattern
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


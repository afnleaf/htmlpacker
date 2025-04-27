/*
#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::{
    //diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    render::{
        mesh::*,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    image::{
        Image,
    },
    scene::{
        SceneRoot
    },
    prelude::*,
};
//use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use std::f32::consts::PI;


// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

pub fn render_earths(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    images: &mut ResMut<Assets<Image>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let textures: Vec<Handle<Image>> = (1..=5)
        .map(|i| asset_server.load(format!("earth_s/texture{i}.png")))
        .collect();
    let shap = meshes.add(Sphere::default().mesh().uv(32, 18));
    let m1 = materials.add(StandardMaterial {
        base_color_texture: Some(textures[0].clone()),
        ..default()
    });
    let m2 = materials.add(StandardMaterial {
        base_color_texture: Some(textures[4].clone()),
        ..default()
    });
    
    // earth 1
    commands.spawn((
        Mesh3d(shap.clone()),
        MeshMaterial3d(m1.clone()),
        Transform::from_xyz(-3.0, 3.0, 5.0)
            .with_scale(Vec3::new(2.0, 2.0, 2.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Shape,
    ));
    // earth 2 
    commands.spawn((
        Mesh3d(shap.clone()),
        MeshMaterial3d(m2.clone()),
        Transform::from_xyz(3.0, 3.0, 5.0)
            .with_scale(Vec3::new(2.0, 2.0, 2.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Shape,
    ));

}

//
pub fn rotate_shapes(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
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

pub fn spawn_shapes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    images: &mut ResMut<Assets<Image>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    // create materials -------------------------------------------------------
    // use asset server to add to materials
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let pyramid_handle = asset_server.load("textures/pyramid.png");
    let p2 = materials.add(StandardMaterial {
        //base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        base_color_texture: Some(pyramid_handle.clone()),
        ..default()
    });

    
    // create meshes ----------------------------------------------------------
    // this can easily be another shape creating function
    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Tetrahedron::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cone::default()),
        meshes.add(ConicalFrustum::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(p2.clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                -Z_EXTENT / 2.,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }
}

// idk --------------------------------------------------------------------
// Load the Fox.glb scene
// The file at 'models/Fox.glb' does not contain the labeled asset 'Scene1'; it contains the following 36 assets: 'Animation0', 'Animation1', 'Animation2', 'Material0', 'Mesh0', 'Mesh0/Primitive0', 'Node0', 'Node1', 'Node10', 'Node11', 'Node12', 'Node13', 'Node14', 'Node15', 'Node16', 'Node17', 'Node18', 'Node19', 'Node2', 'Node20', 'Node21', 'Node22', 'Node23', 'Node24', 'Node25', 'Node3', 'Node4', 'Node5', 'Node6', 'Node7', 'Node8', 'Node9', 'Scene0', 'Skin0', 'Skin0/InverseBindMatrices', 'Texture0'
pub fn spawn_fox(
    commands: &mut Commands,
    asset_server: Res<AssetServer>
) {
    let fox_handle: Handle<Scene> = asset_server.load("models/Fox.glb#Scene0");
    commands.spawn((
        SceneRoot(fox_handle),
        Transform::from_xyz(0.0, 0.0, -5.0)
            .with_scale(Vec3::new(0.07, 0.07, 0.07)),
        GlobalTransform::default(),
    ));
}
*/

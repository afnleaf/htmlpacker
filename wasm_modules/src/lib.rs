#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#![feature(trivial_bounds)]
use wasm_bindgen::prelude::*;
use bevy::{
    color::palettes::basic::SILVER,
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
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

mod dom;
mod tools;
mod scene;
mod earth;
mod trackball_camera;
//mod camera;

// entry point for WASM
#[wasm_bindgen(start)]
pub fn start() {
    // panic hook = better error messages
    console_error_panic_hook::set_once();
    // log start point
    web_sys::console::log_1(&"Starting Bevy WASM application".into());
    // create canvas and add to document
    dom::create_canvas().expect("Failed to create canvas");
    // start app
    start_bevy();
}

pub fn start_bevy() {
    // initialize Bevy
    let mut app = App::new();

    // embed all files in assets folder into the binary
    // this replaces the default bevy asset plugin:5
    app.add_plugins(EmbeddedAssetPlugin {
        mode: PluginMode::ReplaceDefault,
    });
    // add default plugins
    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // use the canvas we just created
                // is there a better way to do this?
                canvas: Some("#canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),
    );
    // add rest
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_systems(
            Startup,
            (
                setup,
                earth::prism_earth,
                //camera::spawn_camera,
                trackball_camera::spawn_trackball_camera
            ),
        )
        //.add_systems(PostStartup, setup2)
        .add_systems(
            Update,
            (
                //scene::rotate_shapes,
                tools::text_update_system, tools::text_color_system,
                trackball_camera::trackball_camera_system
                    .run_if(any_with_component::<trackball_camera::TrackballState>),
                //camera::pan_orbit_camera
                //    .run_if(any_with_component::<camera::PanOrbitState>),
                //update_elevation_instances
                //change_textures,
            ),
        )
        .run();
}

/*
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
*/
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    //prism_earth(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);
    //scene::spawn_fox(&mut commands, asset_server);
    //scene::spawn_shapes(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);
    //scene::render_earths(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);

    // asset loading ----------------------------------------------------------
    // pass asset_server down and do this in another function
    
    // ground -----------------------------------------------------------------
    //ground_plane(&mut commands, &mut meshes, &mut materials);

    lights(&mut commands);
    //camera(&mut commands);
    //camera::spawn_camera(&mut commands);
    //action
    tools::ui(&mut commands);
}



fn ground_plane(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    //materials: Handle<StandardMaterial>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));
}


fn lights(commands: &mut Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(-16.0, 0.0, -16.0),
    ));
    
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(16.0, 0.0, 16.0),
    ));
}

fn camera(commands: &mut Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}


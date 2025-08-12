/*
lib.rs

where we start our bevy app and browser interop
*/

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![feature(trivial_bounds)]

use wasm_bindgen::prelude::*;
use bevy::{
    prelude::*,
    color::palettes::basic::SILVER,
    time::{Timer, TimerMode},
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

use crate::mapupdate::KeyRepeatTimer;

mod dom;
mod tools;
mod earth;
mod sun;
mod orbit_camera;
mod instance_pipeline;
mod mapupdate;


//#[cfg(not(target_family = "wasm"))]
//use bevy_dylib;

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
    // disable right click
    //document.addEventListener('contextmenu', event => event.preventDefault());
    //todo!();
    // start app
    start_bevy();
}

pub fn start_bevy() {
    // initialize Bevy
    let mut app = App::new();
    
    // PLUGINS
    // embed all files in assets folder into the binary
    // this replaces the default bevy asset plugin
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
    // our custom render pipeline that enables instanced geometry
    app.add_plugins(instance_pipeline::CustomMaterialPlugin);
    
    //web_sys::console::log_1(&"TEST 1".into());
    // add rest
    app.insert_resource(KeyRepeatTimer(
        Timer::from_seconds(0.09, TimerMode::Repeating)));
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0))); // black bg
    //web_sys::console::log_1(&"TEST 2".into());
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    //web_sys::console::log_1(&"TEST 3".into());
    app.init_resource::<crate::mapupdate::CurrentMap>();
    
    app.add_systems(Startup,(
        sun::spawn_sun_geocentrism,
        (
            earth::load_elevation_buffers,
            earth::setup_instance_geometries,
        ).chain(),
        initial_setup,
    ).chain());

    //web_sys::console::log_1(&"TEST 4".into());
    app.add_systems(PostStartup,
                orbit_camera::spawn_orbit_camera,
        );
    //web_sys::console::log_1(&"TEST 5".into());
    app.add_systems(
            Update,
            (
                tools::fps_update_system,
                orbit_camera::orbit_camera_system
                    .run_if(any_with_component::<orbit_camera::OrbitState>),
                sun::orbit_geocentrism,
                (
                    mapupdate::map_update_system,
                    mapupdate::update_map_text_display,
                ).chain(),
            ),
        );
    //web_sys::console::log_1(&"TEST 6".into());
    app.run();
    //web_sys::console::log_1(&"IDK WHAT ISNT WORKING".into());
}

// set some base stuff in the scene
fn initial_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // ground -----------------------------------------------------------------
    //ground_plane(&mut commands, &mut meshes, &mut materials);
    //sphere(&mut commands, &mut meshes, &mut materials);
    
    sun::ambient_light(&mut commands);
    tools::fps_widget(&mut commands);
    mapupdate::current_map_widget(&mut commands);
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

fn sphere(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    //materials: Handle<StandardMaterial>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(6.378))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 1.0, 0.0, 0.3), // Semi-transparent green
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}


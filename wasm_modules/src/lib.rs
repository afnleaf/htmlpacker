#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#![feature(trivial_bounds)]
use wasm_bindgen::prelude::*;
use bevy::{
    color::palettes::css::*,
    color::palettes::basic::SILVER,
    //diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::{
        //CascadeShadowConfigBuilder, 
        NotShadowCaster, 
        //NotShadowReceiver
    },
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
use std::f32::consts::PI;

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
    app
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup,(
                setup,
                //earth::prism_earth,
                earth::earth_terrain_mesh,
                //camera::spawn_camera,
                trackball_camera::spawn_trackball_camera
        ))
        //.add_systems(PostStartup, setup2)
        .add_systems(
            Update,
            (
                //scene::rotate_shapes,
                tools::fps_update_system, //tools::text_color_system,
                trackball_camera::trackball_camera_system
                    .run_if(any_with_component::<trackball_camera::TrackballState>),
                geocentrism,
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
    //scene::spawn_fox(&mut commands, asset_server);
    //scene::spawn_shapes(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);
    //scene::render_earths(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);
    
    // ground -----------------------------------------------------------------
    //ground_plane(&mut commands, &mut meshes, &mut materials);
    
    spawn_sun(&mut commands, &mut meshes, &mut materials);
    //lights(&mut commands);
    //camera(&mut commands);
    //camera::spawn_camera(&mut commands);
    //action
    tools::fps_widget(&mut commands);
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

    /*
    commands.insert_resource(AmbientLight {
        //color: WHITE.into(),
        brightness: 1000.0,
        ..default()
    });
    */

    /*
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
    */

#[derive(Component)]
pub struct Star;

#[derive(Component)]
pub struct Orbit {
    pub speed: f32,
    pub axis: Vec3,
    pub center: Vec3,
}

fn spawn_sun(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Use a smaller radius for easier visual confirmation via shadows
    let initial_light_position = Vec3::new(10000.0, 0.0, 0.0);
    let target_point = Vec3::ZERO; // Center of orbit (where the earth is)
    let up_direction = Vec3::Y;
    
    commands.spawn((
        // The light source itself
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 2500.0, // Make it strong enough to see shadows clearly
            shadows_enabled: true,
            shadow_depth_bias: DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
            ..default()
        },
        // Its position and initial orientation
        Transform::from_translation(initial_light_position)
            .looking_at(target_point, up_direction),
        // Marker component for the system to find it
        Star,
        Orbit {
            speed: PI / 64.0,
            axis: Vec3::Y,
            center: target_point,
        },
    ));

    // Spawn a yellow sphere at the same position to visualize the light
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(100.0).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
                base_color: YELLOW.into(),
                emissive: YELLOW.into(),// Make it glow
                ..default()
            })),
        NotShadowCaster,
        Transform::from_translation(initial_light_position),
        Star,
        // Link it to the same orbit behavior
        Orbit {
            speed: PI / 64.0,
            axis: Vec3::Y,
            center: target_point,
        },
    ));
}
        /*
        // Mesh component - a sphere
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Sphere { radius: 0.3 })),
            material: materials.add(StandardMaterial {
                base_color: Color::YELLOW,
                emissive: Color::YELLOW * 2.0, // Make it glow
                ..default()
            }),
            transform: Transform::from_translation(initial_light_position),
            ..default()
        },
        */

fn geocentrism(
    mut query: Query<(&mut Transform, &Orbit), With<Star>>,
    time: Res<Time>,
) {
    for (mut transform, orbit) in &mut query {
        // Use total elapsed time instead of delta time for smoother and more predictable movement
        let total_time = time.elapsed_secs();
        
        // Calculate new position directly using sine and cosine
        let radius = 1000.0;
        let angle = orbit.speed * total_time; // Try a higher speed value like 5.0
        
        // Calculate new position on the circle
        let new_x = radius * angle.cos();
        let new_z = radius * angle.sin();
        
        // Set the new position directly
        transform.translation = Vec3::new(new_x, 0.0, new_z);
        
        // Make sure to point toward the center
        transform.look_at(Vec3::ZERO, Vec3::Y);
        
        //println!("Light at ({}, 0, {}), angle: {}", new_x, new_z, angle);
    }
}

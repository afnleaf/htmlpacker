#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![feature(trivial_bounds)]

use wasm_bindgen::prelude::*;
use bevy::{
    color::palettes::basic::SILVER,
    render::{
        mesh::*,
    },
    image::{Image},
    prelude::*,
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

mod dom;
mod tools;
mod earth;
mod sun;
mod trackball_camera;
mod instancetest;

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

    app.add_plugins(instancetest::CustomMaterialPlugin);
    
    //web_sys::console::log_1(&"TEST 1".into());
    // add rest
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0))); // black bg
    //web_sys::console::log_1(&"TEST 2".into());
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    //web_sys::console::log_1(&"TEST 3".into());
    app.init_resource::<CurrentMap>();
    /*
    app.add_systems(Startup,(
                earth::load_map_data.before(initial_setup),
                earth::prism_earth,
                sun::spawn_sun_geocentrism
                //earth::earth_terrain_mesh,
        ));
    */
    app.add_systems(Startup,(
        earth::load_elevation_buffers,
        earth::generate_static_geometry,
        initial_setup,
        //earth::prism_earth,
        //earth::earth_terrain_mesh,
        //earth::spawn_static_geometry,
        sun::spawn_sun_geocentrism,
        instancetest::setup,
    ).chain());
    //web_sys::console::log_1(&"TEST 4".into());
    app.add_systems(PostStartup,
                //camera::spawn_camera,
                trackball_camera::spawn_trackball_camera,
        );
    //web_sys::console::log_1(&"TEST 5".into());
    app.add_systems(
            Update,
            (
                //scene::rotate_shapes,
                tools::fps_update_system, //tools::text_color_system,
                trackball_camera::trackball_camera_system
                    .run_if(any_with_component::<trackball_camera::TrackballState>),
                sun::orbit_geocentrism,
                map_update_system,
                //earth::update_earth,
                //camera::pan_orbit_camera
                //    .run_if(any_with_component::<camera::PanOrbitState>),
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
    images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // ground -----------------------------------------------------------------
    //ground_plane(&mut commands, &mut meshes, &mut materials);
    
    //spawn_sun(&mut commands, &mut meshes, &mut materials);
    //lights(&mut commands);
    //camera(&mut commands);
    //camera::spawn_camera(&mut commands);
    //action
    tools::fps_widget(&mut commands);
    sun::ambient_light(&mut commands);
    current_map_widget(&mut commands);
}


#[derive(Component)]
pub struct CurrentMapText;

#[derive(Resource)]
pub struct CurrentMap {
    index: usize,
}

impl Default for CurrentMap {
    fn default() -> Self {
        Self {
            index: 0,
        }
    }
}

pub fn current_map_widget(
    commands: &mut Commands,
) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(5.0),
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(SILVER.into()),
        CurrentMapText,
    ));
}

pub fn map_update_system(
    mut current_map: ResMut<CurrentMap>,
    kbd: Res<ButtonInput<KeyCode>>,
    //mut query: Query<&mut TextSpan, With<CurrentMapText>>,
    mut query: Query<&mut TextSpan, With<CurrentMapText>>,
) {
    for mut span in &mut query {
        if kbd.just_pressed(KeyCode::ArrowLeft) {
            current_map.index = (current_map.index + 1).min(108);
        }
        if kbd.just_pressed(KeyCode::ArrowRight) {
            current_map.index = if current_map.index > 1 { current_map.index - 1 } else { 0 };
        }
        **span = format!("{}", current_map.index);
        // render new map
        
    }
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










    //scene::spawn_fox(&mut commands, asset_server);
    //scene::spawn_shapes(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);
    //scene::render_earths(&mut commands, &mut meshes, &mut images, &mut materials, asset_server);

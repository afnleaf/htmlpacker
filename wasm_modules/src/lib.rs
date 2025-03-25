use wasm_bindgen::prelude::*;
use web_sys;
//use base64::prelude::*;
use std::f32::consts::PI;
//use std::time::Duration;
//use iyes_perf_ui::prelude::*;
//use bevy_mini_fps::fps_plugin;
use bevy::{
    color::palettes::basic::SILVER,
    color::palettes::css::GOLD,
    //diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    //ui::TextStyle,
    //diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    //window::WindowResized
};

mod dom;

// Entry point for WebAssembly
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
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // use the canvas we just created
                // is there a better way to do this?
                canvas: Some("#canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        //.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        //.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        //.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        //.add_plugins(FrameTimeDiagnosticsPlugin) 
        //.add_plugins(PerfUiPlugin)
        //.add_plugins(fps_plugin!())
        
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate,
                text_update_system, text_color_system
                //on_resize,
            ),
        )
        .run();
}

// need a font system


// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // create materials -------------------------------------------------------
    // load a texture and retrieve its aspect ratio
    //let texture_handle = asset_server.load("textures/array_texture.png");
    let pyramid_handle = asset_server.load("textures/pyramid.png");
    
  
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });


    let p2 = materials.add(StandardMaterial {
        //base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        base_color_texture: Some(pyramid_handle.clone()),
        ..default()
    });

    // loading image from embedded base64
    /* 
    // load at runtime
    // Create a new image (64x64 red square)
    let mut image_data = Vec::with_capacity(64 * 64 * 4);
    for _ in 0..(64 * 64) {
        // RGBA: Red, fully opaque
        image_data.extend_from_slice(&[255, 0, 0, 255]);
    }
    
    let image = Image::new(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_data,
        TextureFormat::Rgba8UnormSrgb,
        ImageSampler::default(),
    );
    
    // Add the image to the asset system and get a handle
    let image_handle = asset_server.add(image);
    
    // Spawn a sprite using the created image
    commands.spawn(SpriteBundle {
        texture: image_handle,
        ..default()
    });


    // If you know the images you need at compile time, a simpler approach is to use Bevy's embedded assets feature:
    // Embed the PNG data directly in the binary
    bevy::asset::embedded_asset!(EMBEDDED_ICON, "path/to/your/icon.png");
    
    // Load the embedded asset using the AssetServer
    let icon_handle = asset_server.load(EMBEDDED_ICON);


    // Decode the base64 data
    let image_data = STANDARD.decode(base64_clean).expect("Failed to decode base64 data");
    
    // Create a new Bevy Image asset from the decoded data
    let image = Image::from_buffer(
        &image_data,
        bevy::render::texture::ImageType::Extension("png"),
        ImageSampler::default(),
        false,
    ).expect("Failed to create image from buffer");

// Add the image to Bevy's asset system
    let image_handle = asset_server.add(image);
    
    // Store the handle somewhere accessible, for example in a resource
    app.world.insert_resource(CustomImageResource(image_handle));

// A resource to store our loaded image handle
#[derive(Resource)]
struct CustomImageResource(Handle<Image>);

    */


    /*
    let aspect = 0.25;
    // create a new quad mesh. this is what we will apply the texture to
    //let quad_width = 8.0;
    //let quad_handle = meshes.add(Rectangle::new(quad_width, quad_width * aspect));
    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    // this material modulates the texture to make it red (and slightly transparent)
    let red_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.5),
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // and lets make this one blue! (and also slightly transparent)
    let blue_material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    */
    
    // create meshes ----------------------------------------------------------
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
    
    // lights -----------------------------------------------------------------
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));


    // ground -----------------------------------------------------------------
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));



    // camera ----------------------------------------------------------------- 
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    // UI ---------------------------------------------------------------------
    // Text with one section
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("hello\nbevy!"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 67.0,
            ..default()
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        ColorText,
    ));

    // Empty text for the parent to satisfy Bevy’s hierarchy
    // WHY, im not sure
    commands
    .spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(5.0),
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(GOLD.into()),
        FpsText,
    ));

    commands.spawn((
        // Here we are able to call the `From` method instead of creating a new `TextSection`.
        // This will use the default font (a minimal subset of FiraMono) and apply the default styling.
        Text::new("From an &str into a Text with the default font!"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
    // end of setup -----------------------------------------------------------
    // how do we split this up while keeping the references intact?
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
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


// FPS COUNTER -------------------------------------------------
// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;

fn text_color_system(time: Res<Time>, mut query: Query<&mut TextColor, With<ColorText>>) {
    for mut text_color in &mut query {
        let seconds = time.elapsed_secs();

        // Update the color of the ColorText span.
        text_color.0 = Color::srgb(
            ops::sin(1.25 * seconds) / 2.0 + 0.5,
            ops::sin(0.75 * seconds) / 2.0 + 0.5,
            ops::sin(0.50 * seconds) / 2.0 + 0.5,
        );
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}

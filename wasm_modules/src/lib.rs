use wasm_bindgen::prelude::*;
use std::f32::consts::PI;
use bevy::{
    color::palettes::basic::SILVER,
    color::palettes::css::GOLD,
    render::{
        mesh::*,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    image::{
        Image,
    },
    scene::{
        SceneRoot
    },
    prelude::*,
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};

mod dom;

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
    // add rest
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate_shapes,
                text_update_system, text_color_system
            ),
        )
        .run();
}



// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

//const TEXTURE_DATA: &[u8] = include_bytes!("../assets/textures/pyramid.png");
//const TEXTURE_DATA: &[u8] = include_bytes!("../assets/earth_l/texture1.png");
//const FOX_MODEL: &[u8] = include_bytes!("../assets/models/Fox.glb");

//fn load_map()

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
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

    let textures: Vec<Handle<Image>> = (1..=109)
        .map(|i| asset_server.load(format!("earth_s/texture{i}.png")))
        .collect();
    let shap = meshes.add(Sphere::default().mesh().uv(32, 18));
    let m1 = materials.add(StandardMaterial {
        base_color_texture: Some(textures[0].clone()),
        ..default()
    });
    let m2 = materials.add(StandardMaterial {
        base_color_texture: Some(textures[108].clone()),
        ..default()
    });

    commands.spawn((
        Mesh3d(shap.clone()),
        MeshMaterial3d(m1.clone()),
        Transform::from_xyz(-3.0, 3.0, 5.0)
            .with_scale(Vec3::new(2.0, 2.0, 2.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Shape,
    ));
    
    commands.spawn((
        Mesh3d(shap.clone()),
        MeshMaterial3d(m2.clone()),
        Transform::from_xyz(3.0, 3.0, 5.0)
            .with_scale(Vec3::new(2.0, 2.0, 2.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Shape,
    ));
    /*
    for i in 1..109 {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(textures[i].clone()),
            ..default()
        });
        commands.spawn((
            Mesh3d(shap.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-5.0+(i as f32), 0.0, -5.0).with_scale(Vec3::new(0.07, 0.07, 0.07)).with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }
    */



    // asset loading ----------------------------------------------------------
    // pass asset_server down and do this in another function
    let pyramid_handle = asset_server.load("textures/pyramid.png");
    // Load the Fox.glb scene
    // The file at 'models/Fox.glb' does not contain the labeled asset 'Scene1'; it contains the following 36 assets: 'Animation0', 'Animation1', 'Animation2', 'Material0', 'Mesh0', 'Mesh0/Primitive0', 'Node0', 'Node1', 'Node10', 'Node11', 'Node12', 'Node13', 'Node14', 'Node15', 'Node16', 'Node17', 'Node18', 'Node19', 'Node2', 'Node20', 'Node21', 'Node22', 'Node23', 'Node24', 'Node25', 'Node3', 'Node4', 'Node5', 'Node6', 'Node7', 'Node8', 'Node9', 'Scene0', 'Skin0', 'Skin0/InverseBindMatrices', 'Texture0'
    let fox_handle: Handle<Scene> = asset_server.load("models/Fox.glb#Scene0");
    commands.spawn((
        SceneRoot(fox_handle),
        Transform::from_xyz(0.0, 0.0, -5.0)
            .with_scale(Vec3::new(0.07, 0.07, 0.07)),
        GlobalTransform::default(),
    ));

    // create materials -------------------------------------------------------
    // use asset server to add to materials
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

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
    


    // ground -----------------------------------------------------------------
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    // lights -----------------------------------------------------------------
    lights(&mut commands);

    // camera ----------------------------------------------------------------- 
    camera(&mut commands);

    // UI ---------------------------------------------------------------------
    ui(&mut commands);
    // end of setup -----------------------------------------------------------
    // how do we split this up while keeping the references intact?
}


fn rotate_shapes(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
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

// systems --------------------------------------------------------------------
fn lights(commands: &mut Commands) {
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
}

fn camera(commands: &mut Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

fn ui(commands: &mut Commands) {
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
    commands.spawn((
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
}
    


// FPS COUNTER ----------------------------------------------------------------
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

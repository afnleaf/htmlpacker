//use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys;
use std::f32::consts::PI;
use bevy::{
    color::palettes::basic::SILVER,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    //window::WindowResized
};



// function to create and set up the canvas element
// inside the dom
fn create_canvas() -> Result<(), JsValue> {
    // window and document DOM
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // create canvas element
    let canvas = document.create_element("canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    
    // window dimensions
    let width = window.inner_width()?.as_f64().unwrap() as u32;
    let height = window.inner_height()?.as_f64().unwrap() as u32;
    
    // Set canvas dimensions and ID
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.set_id("canvas");
    
    // should this be in a css file?
    // no we don't need css besides this
    canvas.style().set_property("display", "block")?;
    canvas.style().set_property("margin", "0")?;
    canvas.style().set_property("padding", "0")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    
    // append canvas to DOM
    document.body().unwrap().append_child(&canvas)?;
    
    // body styles full screen
    document.body().unwrap().style().set_property("margin", "0")?;
    document.body().unwrap().style().set_property("padding", "0")?;
    document.body().unwrap().style().set_property("overflow", "hidden")?;
    document.body().unwrap().style().set_property("width", "100vw")?;
    document.body().unwrap().style().set_property("height", "100vh")?;
    
    // console log allows for debugging in browser
    web_sys::console::log_1(&"Canvas created successfully".into());
    
    Ok(())
}

// Entry point for WebAssembly
#[wasm_bindgen(start)]
pub fn start() {
    // panic hook = better error messages
    console_error_panic_hook::set_once();
    // log start point
    web_sys::console::log_1(&"Starting Bevy WASM application".into());
    // create canvas and add to document
    create_canvas().expect("Failed to create canvas");
    
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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                rotate,
                //on_resize,
            ),
        )
        .run();
}



//not work
/*
fn on_resize(
    mut window: Single<&mut Window>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for e in resize_reader.read() {
        // When resolution is being changed
        //
        web_sys::console::log_1(&"TEST--------------".into());
        window.resolution.set(e.width, e.height);
        web_sys::console::log_1(&format!("w{} {}", e.width, e.height).into());

    }

}
*/



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
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

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
            MeshMaterial3d(debug_material.clone()),
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

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

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


/*
use std::f64;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn start() {
    // get DOM document from window
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
   
    //let canvas = document.get_element_by_id("canvas").unwrap();
    //let canvas: web_sys::HtmlCanvasElement = canvas
    //    .dyn_into::<web_sys::HtmlCanvasElement>()
    //    .map_err(|_| ())
    //    .unwrap();
    
    // create a canvas
    let canvas = document.create_element("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    // get window dimensions
    let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    let height = window.inner_height().unwrap().as_f64().unwrap() as u32;

    // set canvas dimensions to match window size
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.set_id("canvas");
    
    // Add some CSS to ensure it fills the screen without scrollbars
    canvas.style().set_property("display", "block").unwrap();
    canvas.style().set_property("margin", "0").unwrap();
    canvas.style().set_property("padding", "0").unwrap();
    
    // append canvas to document body
    document.body().unwrap().append_child(&canvas).unwrap();
    
    // Also set the body and html styles to remove margins and prevent scrollbars
    document.body().unwrap().style().set_property("margin", "0").unwrap();
    document.body().unwrap().style().set_property("overflow", "hidden").unwrap();
    
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();


    context.begin_path();

    // Draw the outer circle.
    context
        .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context
        .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context
        .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    context.stroke();
}
*/

// The 3D scene setup function
/*
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Log that we're setting up the scene
    web_sys::console::log_1(&"Setting up 3D scene".into());

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // tetrahedron
    commands.spawn((
        Mesh3d(meshes.add(Tetrahedron::default())),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(ConicalFrustum::default())),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 0))),
        Transform::from_xyz(2.0, 2.0, 0.0),
    ));
    
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 20.0, 4.0),
    ));
    
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Log that scene setup is complete
    web_sys::console::log_1(&"3D scene setup complete".into());
}
*/

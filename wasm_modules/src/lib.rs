use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys;

// Function to create and set up the canvas element
fn create_canvas() -> Result<(), JsValue> {
    // Get window and document
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // Create canvas element
    let canvas = document.create_element("canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    
    // Get window dimensions
    let width = window.inner_width()?.as_f64().unwrap() as u32;
    let height = window.inner_height()?.as_f64().unwrap() as u32;
    
    // Set canvas dimensions and ID
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.set_id("canvas");
    
    // Add CSS styling
    canvas.style().set_property("display", "block")?;
    canvas.style().set_property("margin", "0")?;
    canvas.style().set_property("padding", "0")?;
    canvas.style().set_property("width", "100%")?;
    canvas.style().set_property("height", "100%")?;
    
    // Append canvas to document body
    document.body().unwrap().append_child(&canvas)?;
    
    // Set body styles
    document.body().unwrap().style().set_property("margin", "0")?;
    document.body().unwrap().style().set_property("padding", "0")?;
    document.body().unwrap().style().set_property("overflow", "hidden")?;
    document.body().unwrap().style().set_property("width", "100vw")?;
    document.body().unwrap().style().set_property("height", "100vh")?;
    
    // Console log for debugging
    web_sys::console::log_1(&"Canvas created successfully".into());
    
    Ok(())
}

// Entry point for WebAssembly
#[wasm_bindgen(start)]
pub fn start() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Log that we're starting
    web_sys::console::log_1(&"Starting Bevy WASM application".into());
    
    // Create the canvas first
    create_canvas().expect("Failed to create canvas");
    
    // Now initialize Bevy
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Use the canvas we just created
                canvas: Some("#canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

// The 3D scene setup function
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
    
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    // Log that scene setup is complete
    web_sys::console::log_1(&"3D scene setup complete".into());
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

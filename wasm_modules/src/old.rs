
/*
use std::f64;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet() -> String {
    "Hello from webassembly!".to_string()
}

#[wasm_bindgen]
pub fn hello() -> String {
    "hello".to_string()
}

#[wasm_bindgen]
pub fn goodbye() -> String {
    "goodbye".to_string()
}
*/

/*
//#[wasm_bindgen]
#[wasm_bindgen(start)]
pub fn run() {
    bare_bones();
    using_a_macro();
    using_web_sys();
}

// First up let's take a look of binding `console.log` manually, without the
// help of `web_sys`. Here we're writing the `#[wasm_bindgen]` annotations
// manually ourselves, and the correctness of our program relies on the
// correctness of these annotations!

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

fn bare_bones() {
    log("Hello from Rust!");
    log_u32(42);
    log_many("Logging", "many values!");
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the Wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn using_a_macro() {
    console_log!("Hello {}!", "world");
    console_log!("Let's print some numbers...");
    console_log!("1 + 3 = {}", 1 + 3);
}

// And finally, we don't even have to define the `log` function ourselves! The
// `web_sys` crate already has it defined for us.

fn using_web_sys() {
    use web_sys::console;

    console::log_1(&"Hello using web-sys".into());

    let js: JsValue = 4.into();
    console::log_2(&"Logging arbitrary values looks like".into(), &js);
}
*/

/*
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

    // set canvas attributes
    canvas.set_width(150);
    canvas.set_height(150);
    canvas.set_id("canvas");

    // append canvas to document body
    document.body().unwrap().append_child(&canvas).unwrap();
    
    
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

/*
#[no_mangle]
pub extern "C" fn greet() -> *mut u8 {
    b"Hello from webassembly!".to_ptr() as *mut u8
}


#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn greeting_code() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// no worky cause pointer to js!!
#[no_mangle]
pub extern "C" fn greet() -> *mut u8 {
    b"Hello from webassembly!".as_ptr() as *mut u8
}
*/

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
    /*
    //let font = asset_server.load("fonts/your_font.ttf");
    let font: Handle<Font> = Default::default();
    // You may need to register the font as the default UI font
    //commands.insert_resource(UiFont(font));
    commands.spawn(TextBundle::from_section(
        "FPS: ...", // Update this text as needed
        TextStyle {
            font,
            font_size: 16.0,
            color: Color::WHITE,
        },
    ));
    //commands.spawn(PerfUiAllEntries::default());
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            values_col_width: 32.0,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
    */

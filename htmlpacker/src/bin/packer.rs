use std::error::Error;
use std::process::Command;
use std::fs;
// crates
// use tokio;
// use reqwest;
// modules
use ::htmlpacker::encoder;
use ::htmlpacker::htmlpacker;

/*
fn build_wasm() {
    Command::new("rustc")
        .args(&[
            "--target", "wasm32-unknown-unknown",
            "-O", 
            "--crate-type=cdylib", 
            "./wasm_modules/src/lib.rs", 
            "-o", "./public/wasm_test.wasm"
        ])
        .status()
        .expect("Failed to compile WebAssembly");
    println!("WebAssembly compiled successfully");
}
*/

/*
fn build_wasm() {
    // Run cargo build in the wasm_modules directory with the correct target
    let status = Command::new("cargo")
        .current_dir("./wasm_modules")
        .args(&[
            "build",
            "--target", "wasm32-unknown-unknown",
            "--release"
        ])
        .status()
        .expect("Failed to compile WebAssembly");
    
    if !status.success() {
        panic!("WebAssembly compilation failed");
    }
    
    // Make sure the public directory exists
    //fs::create_dir_all("./public").expect("Failed to create public directory");
    
    // Copy the compiled wasm file to where the main project expects it
    fs::copy(
        "./wasm_modules/target/wasm32-unknown-unknown/release/wasm_modules.wasm",
        "./public/wasm_test.wasm"
    ).expect("Failed to copy WebAssembly file");
    
    println!("WebAssembly compiled successfully");
}
*/

//wasm-pack build --target no-modules
fn build_wasm() {
    Command::new("wasm-pack")
        .current_dir("../wasm_modules")
        .args(&[
            "build",
            "--target",
            "no-modules",
        ])
        .status()
        .expect("Failed to compile WebAssembly");
    println!("WebAssembly compiled successfully");
}

// async !!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    build_wasm();

    let css_urls = vec![
        "https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css"
    ];

    let external_scripts_list = vec![
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/examples/js/controls/TrackballControls.min.js"
    ];

    let local_scripts_list = vec![
        "../public/script.js",
        "../wasm_modules/pkg/wasm_modules.js"
    ];

    // external css
    let css_text = htmlpacker::get_css_string(css_urls).await?;

    // external scripts to fetch
    let external_scripts_text = htmlpacker::get_external_scripts_text(
                                    external_scripts_list).await?;

    // get local scripts
    let local_scripts_text = htmlpacker::get_local_scripts_text(local_scripts_list)?;
    //println!("{:?}", external_scripts_text);
    //println!("{:?}", local_scripts_text);
    //

    // base64 encoder
    let text64 = encoder::encode_text_base64("hello world!!!\nt. packer");
    let png64 = encoder::encode_png_base64("../public/wizard.png")?;
    //let wasm64 = encoder::encode_wasm_base64("../public/wasm_test.wasm")?;
    let wasm64 = encoder::encode_wasm_base64("../wasm_modules/pkg/wasm_modules_bg.wasm")?;
    //println!("{:#?}", &text64);
    //println!("{:#?}", &png64);
    let bin: Vec<encoder::Base> = vec![text64, png64, wasm64];


    let markup = htmlpacker::page(
        css_text,
        external_scripts_text,
        local_scripts_text,
        bin,
    );

    let html = markup.into_string();
    //println!("{}", html);
    htmlpacker::save_html(html)?;

    Ok(())
}


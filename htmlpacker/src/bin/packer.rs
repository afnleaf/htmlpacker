/*
This is the main of the program that packs everything together

we fetch scripts from here
might turn into some type of cli tool that you use with yaml files

local, external, css, text, png
pack it up

compress it with brotli
then encode it in base64
*/

use std::error::Error;
use std::process::Command;
use std::path::Path;
//use std::fs;
// crates
// use tokio;
// use reqwest;
// modules
use ::htmlpacker::encoder;
use ::htmlpacker::htmlpacker;


//wasm-pack build --target no-modules
fn build_wasm(dir: &str) -> Result<(), Box<dyn Error>> {
    println!("Building WASM in {}", dir);
    Command::new("wasm-pack")
        .current_dir(dir)
        .args(&[
            "build",
            "--target",
            "no-modules",
        ])
        .status()
        .expect("Failed to compile WASM.");
    println!("WASM compiled.");

    Ok(())
}

// async !!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // make sure to compile our wasm binaries and js glue first
    build_wasm("../wasm_decoder").unwrap();
    build_wasm("../wasm_modules").unwrap();

    // metadata
    let css_urls = vec![
        //"https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css"
    ];

    // doesnt work need to create my own non module version?
    let external_scripts_list = vec![
        //"https://cdn.jsdelivr.net/npm/brotli-compress@1.3.3/index.min.js",
        //"https://cdnjs.cloudflare.com/ajax/libs/brotli/1.3.2/decode.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/examples/js/controls/TrackballControls.min.js"
    ];

    let local_scripts_list = vec![
        "../wasm_decoder/pkg/wasm_decoder.js",
        "../wasm_modules/pkg/wasm_modules.js", // bindgen
        "../public/decoder.js", // decode wasm from base64
        //"../public/app.js", // app logic
        //"../public/script.js", // test script
        "../public/app2.js",
    ];

    // external css
    let css_text = htmlpacker::get_css_string(css_urls).await?;

    // favicon processing
    let icon_path = "../public/icon.svg";
    let icon_svg = htmlpacker::get_local_script(Path::new(icon_path)).unwrap();
    let icon_svg64 = encoder::encode_icon_svg_base64(icon_svg);
    //println!("{:?}", icon_svg64);
    let icons = vec![icon_svg64];

    // external scripts to fetch
    let external_scripts_text = htmlpacker::get_external_scripts_text(
        external_scripts_list).await?;

    // get local scripts
    let mut local_scripts_text = htmlpacker::get_local_scripts_text(
        local_scripts_list)?;
    //println!("{:?}", external_scripts_text);
    //println!("{:?}", local_scripts_text);
    // rename wasm_bindgen
    //local_scripts_text[0] = rename_wasm_bindgen(local_scripts_text[0], "poop"); 
    local_scripts_text[0] = local_scripts_text[0].replace(
        "wasm_bindgen",
        "wasm_decoder");

    // base64 encoder
    //let text64 = encoder::encode_text_base64("hello world!!!\nt. packer", "bin-text");
    //let png64 = encoder::encode_png_base64("../public/wizard.png", "bin-png")?;
    //let wasm64 = encoder::encode_wasm_base94("../wasm_modules/pkg/wasm_modules_bg.wasm", "bin-wasm")?;
    // this decoder is the reason we can do brotli decompress
    let wasm_decoder64 = encoder::encode_wasm_base64(
        "../wasm_decoder/pkg/wasm_decoder_bg.wasm", 
        "bin-wasm-decoder")?;
    let wasm_module64 = encoder::encode_wasm_base64_brotli(
        "../wasm_modules/pkg/wasm_modules_bg.wasm", 
        "bin-wasm")?;
    //let wasm64 = encoder::encode_wasm_base64("../public/wasm_test.wasm")?;
    //println!("{:#?}", &text64);
    //println!("{:#?}", &png64);
    let bin: Vec<encoder::Base> = vec![
        //text64,
        //png64,
        wasm_decoder64,
        wasm_module64
    ];

    let markup = htmlpacker::page(
        css_text,
        icons,
        external_scripts_text,
        local_scripts_text,
        bin,
    );

    let html = markup.into_string();
    //println!("{}", html);
    htmlpacker::save_html(html)?;

    Ok(())
}

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

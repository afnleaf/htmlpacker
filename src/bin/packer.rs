use std::error::Error;
use std::process::Command;
//use std::fs;
// crates
// use tokio;
// use reqwest;
// modules
use ::htmlpacker::encoder;
use ::htmlpacker::htmlpacker;


//wasm-pack build --target no-modules
fn build_wasm() -> Result<(), Box<dyn Error>> {
    Command::new("wasm-pack")
        .current_dir("../wasm_modules")
        .args(&[
            "build",
            "--target",
            "no-modules",
        ])
        .status()
        .expect("WASM failed to compile.");
        //panic?
    /* 
    // make sure the public directory exists
    fs::create_dir_all("./public").expect("Failed to create public directory");
    
    // Copy the compiled wasm file to where the main project expects it
    fs::copy(
        "./wasm_modules/target/wasm32-unknown-unknown/release/wasm_modules.wasm",
        "./public/wasm_test.wasm"
    ).expect("Failed to copy WebAssembly file");
    */
    println!("WASM compiled.");

    Ok(())
}

// async !!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //println!("TEST");

    build_wasm();

    let css_urls = vec![
        "https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css"
    ];

    let external_scripts_list = vec![
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/examples/js/controls/TrackballControls.min.js"
    ];
    
    println!("TEST");

    let local_scripts_list = vec![
        "../wasm_modules/pkg/wasm_modules.js", // bindgen
        "../public/decoder.js", // decode wasm
        "../public/script.js", // app logic
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
    let png64 = encoder::encode_png_base64("./public/wizard.png")?;
    let wasm64 = encoder::encode_wasm_base64("./wasm_modules/pkg/wasm_modules_bg.wasm")?;
    //let wasm64 = encoder::encode_wasm_base64("./public/wasm_test.wasm")?;
    //println!("{:#?}", &text64);
    //println!("{:#?}", &png64);
    let bin: Vec<encoder::Base> = vec![text64, png64, wasm64];


    println!("{:?}", local_scripts_text);
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


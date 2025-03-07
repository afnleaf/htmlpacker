use std::error::Error;
// crates
use base64::prelude::*;
// use tokio;
// use reqwest;
// modules
mod htmlpacker;

fn encode_base64(s: &str) -> String {
    let encoded = BASE64_STANDARD.encode(s.as_bytes());

    println!("text: {}", s);
    println!("b64:  {}", &encoded);

    encoded
}

// async !!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let css_urls = vec![
        "https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css"
    ];

    let external_scripts_list = vec![
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/examples/js/controls/TrackballControls.min.js"
    ];

    let local_scripts_list = vec![
        "./public/script.js",
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
    let b64 = encode_base64("hello world!!!");
    println!("{}", b64);

    let markup = htmlpacker::page(
        css_text,
        external_scripts_text,
        local_scripts_text,
        b64,
    );

    let html = markup.into_string();
    //println!("{}", html);
    htmlpacker::save_html(html)?;

    Ok(())
}


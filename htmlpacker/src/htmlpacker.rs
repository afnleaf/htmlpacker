use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
// crates
use maud::{DOCTYPE, html, Markup, PreEscaped};
// use tokio;
// use reqwest;
//
use crate::encoder::Base;

//use htmlpacker::encoder;

//#[derive(Debug)]
//struct Embed {
//    //local css
//    //external css
//    //local scripts
//    //external scripts
//    //what else
//}

//<!-- For older browsers that don't support WASM natively -->
//<script src="https://cdn.jsdelivr.net/npm/wasm-polyfill/wasm-polyfill.min.js"></script>
// hmmm

// head part of html
fn head(
    css: String,
    icons: Vec<String>,
) -> Markup {
    let viewport = concat!(
        "width=device-width, ",
        "initial-scale=1.0, ",
        "maximum-scale=1.0, ",
        "user-scalable=1"
    );
    html! {
        (DOCTYPE)
        meta charset = "utf-8";
        meta name = "viewport" content = (viewport);
        //meta name = "description" content = "htmlpacked webapp"
        //meta name = "author" content = "htmlpacker"
        title { "htmlpacker" }
        (favicons(icons))
        style { (css) }
    }
}

fn favicons(icons: Vec<String>) -> Markup {
    html! {
        // basic - covers most needs
        //link rel="icon" type="image/x-icon" href="data:image/x-icon;base64,YOUR_ICO_BASE64_HERE";
        
        // modern browsers - SVG favicon (best option when available)
        //link rel="icon" type="image/svg+xml" href="data:image/svg+xml;base64,(icons[0])";
        link rel="icon" type="image/svg+xml" href=(format!("data:image/svg+xml;base64,{}", icons[0]));
        
        // fallback PNGs for various sizes
        //link rel="icon" type="image/png" sizes="16x16" href="data:image/png;base64,YOUR_16x16_PNG_BASE64_HERE";
        //link rel="icon" type="image/png" sizes="32x32" href="data:image/png;base64,YOUR_32x32_PNG_BASE64_HERE";
        
        // iOS/macOS support
        //link rel="apple-touch-icon" sizes="180x180" href="data:image/png;base64,YOUR_180x180_PNG_BASE64_HERE";
    }
}

// replace problematic characters
// tokens to watch for:
// &amp; -> &
// &lt; -> <
// &gt; -> >
// && getting encoded
// </script> appearing in strings
// unexpected semicolons from minification
// PreEscaped does this for us.

// place a bunch of scripts
fn scripts(
    external_scripts_text: Vec<String>,
    local_scripts_text: Vec<String>,
) -> Markup {
    html! {
        // you can do for loops in here :o
        // external scripts
        @for script in &external_scripts_text {
            script {
                (PreEscaped(script))
            }
        }
        // local scripts
        @for script in &local_scripts_text {
            script {
                (PreEscaped(script))
            }
        }
    }
}

fn binary(
    bin: Vec<Base>,
) -> Markup {
    html! {
        @for b in &bin {
            pre id=(b.id) style="display: none;" {
                (b.sl)
            }
        }
    }
}

// head
// body
// scripts
// combine into page
pub fn page(
    css: String,
    icons: Vec<String>,
    external_scripts_text: Vec<String>,
    local_scripts_text: Vec<String>,
    bin: Vec<Base>,
)
-> Markup {
    html! {
        (head(css, icons))
        body { 
            (binary(
                bin,
            ))
            (scripts(
                external_scripts_text,
                local_scripts_text,
            ))
        }
    }
}


// go through each external script file
// saving 
pub async fn get_external_scripts_text(
    script_urls: Vec<&str>
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut script_strings: Vec<String> = vec![];
    for url in script_urls {
        script_strings.push(reqwest::get(url).await?.text().await?);
    }
    Ok(script_strings) 
}

// go through each local script file
pub fn get_local_scripts_text(
    script_path_literals: Vec<&str>
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut script_strings: Vec<String> = vec![];
    for path in script_path_literals {
        script_strings.push(
            get_local_script(Path::new(path)).unwrap()
        )
    }
    Ok(script_strings)
}

pub fn get_local_script(path: &Path) -> Result<String, Box<dyn Error>>{
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

// append each external css file together
pub async fn get_css_string(
    css_urls: Vec<&str>
) -> Result<String, Box<dyn Error>> {
    let mut css_string = String::from(""); 
    for url in css_urls {
        css_string += &reqwest::get(url).await?.text().await?;
    }
    Ok(css_string)
}

// save our string to an html file
pub fn save_html(html: String) -> Result<(), Box<dyn Error>> {
    // create the directory and all its parent directories if they don't exist
    let output_dir = Path::new("../output");
    fs::create_dir_all(output_dir)?;

    let mut file = File::create(output_dir.join("index.html"))?;

    file.write_all(html.as_bytes())?;
    Ok(())
}


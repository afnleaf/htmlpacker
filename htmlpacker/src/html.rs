/*
* html.rs
*
* using the maud templating crate 
* pack the sources into their part of the html
*/

use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::path::{PathBuf};
use std::error::Error;
// crates
use maud::{DOCTYPE, html, Markup, PreEscaped};
// local
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
        "\n"
        meta charset = "utf-8";
        "\n"
        meta name = "viewport" content = (viewport);
        //meta name = "description" content = "htmlpacked webapp"
        //meta name = "author" content = "htmlpacker"
        title { "htmlpacker" }
        "\n"
        (favicons(icons))
        "\n"
        style { "\n"(css)"\n" }
        "\n"
    }
}

fn favicons(icons: Vec<String>) -> Markup {
    if icons.len() > 0 {
        html! {
            // basic - covers most needs
            //link rel="icon" type="image/x-icon" href="data:image/x-icon;base64,YOUR_ICO_BASE64_HERE";
            
            // modern browsers - SVG favicon (best option when available)
            //link rel="icon" type="image/svg+xml" href="data:image/svg+xml;base64,(icons[0])";

            "\n"
            link rel="icon" type="image/svg+xml" href=(format!("data:image/svg+xml;base64,{}", icons[0]));
            "\n"
            // fallback PNGs for various sizes
            //link rel="icon" type="image/png" sizes="16x16" href="data:image/png;base64,YOUR_16x16_PNG_BASE64_HERE";
            //link rel="icon" type="image/png" sizes="32x32" href="data:image/png;base64,YOUR_32x32_PNG_BASE64_HERE";
            
            // iOS/macOS support
            //link rel="apple-touch-icon" sizes="180x180" href="data:image/png;base64,YOUR_180x180_PNG_BASE64_HERE";
        }
    } else {
        html! {
            "\n"
        }
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

// place a bunch of html text
fn place_html_texts(
    text: Vec<String>,
) -> Markup {
    html! {
        @for t in &text {
            "\n"
            (PreEscaped(t))
            "\n"
        }
    }
}

// place a bunch of scripts
fn scripts(
    //external_scripts_text: Vec<String>,
    //local_scripts_text: Vec<String>,
    js: Vec<String>
) -> Markup {
    html! {
        "\n"
        // you can do for loops in here :o
        @for script in &js {
            script {
                "\n"
                (PreEscaped(script))
                "\n"
            }
            "\n"
        }

        // external scripts
        //@for script in &external_scripts_text {
        //    script {
        //        (PreEscaped(script))
        //    }
        //}
        //// local scripts
        //@for script in &local_scripts_text {
        //    script {
        //        (PreEscaped(script))
        //    }
        //}
    }
}

fn binary(
    bin: Vec<Base>,
) -> Markup {
    html! {
        "\n"
        @for b in &bin {
            pre id=(b.id) hash=(b.hash) style="display: none;" {
                "\n"
                (b.text)
                "\n"
            }
            "\n"
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
    html_texts: Vec<String>,
    js: Vec<String>,
    //external_scripts_text: Vec<String>,
    //local_scripts_text: Vec<String>,
    bin: Vec<Base>,
)
-> Markup {
    html! {
        (head(css, icons))
        "\n"
        body {
            "\n"
            (binary(
                bin,
            ))
            "\n"
            (scripts(
                js
            ))
            "\n"
            (place_html_texts(
                html_texts
            ))
            "\n"
            //(scripts(
            //    external_scripts_text,
            //    local_scripts_text,
            //))
        }
        "\n"
    }
}

// save our string to an html file
pub fn save_html(
    html: String,
    output: PathBuf,
) -> Result<(), Box<dyn Error>> {
    /*
    let output_dir = Path::new("../output");
    fs::create_dir_all(output_dir)?;
    */
    // create the directory and all its parent directories if they don't exist
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    //let mut file = File::create(output_dir.join("index.html"))?;
    let mut file = File::create(output)?;
    file.write_all(html.as_bytes())?;
    
    Ok(())
}


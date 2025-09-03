/*
* packer.rs
*
* the main packing logic
*/
use crate::config::{AssetSource, WasmModule, CompressionType};
use crate::cli::{YamlRoot, Cli};
use crate::encoder::{*, Base};
use crate::encoder;
use crate::wasmbuilder;
use crate::html;
use crate::fetcher;

//
use std::error::Error;
use std::fs;

use clap::{Parser};

// read yaml from file
// set config from yaml
// need serde structs

pub async fn pack() -> Result<(), Box<dyn Error>> {
    // parse CLI
    let cli = Cli::parse();
    println!("Config: {}", cli.config.display());
    println!("Output: {}", cli.output.display());
    
    // make sure to compile our wasm binaries and js glue first
    // how to disable this if already done?
    wasmbuilder::compile_wasm_modules().await?;


    //let yaml_text = fs::read_to_string("./test.yaml")?;
    let yaml_text = fs::read_to_string(cli.config)?;
    let yaml_root: YamlRoot = serde_yaml::from_str(&yaml_text)?;
    println!("extracted yaml");
    //println!("{:?}", yaml_text);
    //println!("{:#?}", &yaml_root.pack);
    let config = crate::cli::set_config_from_yaml(yaml_root.pack).await?;
    println!("loaded config from yaml");
    
    // favicon multiple but one supported rn
    // SLOP
    let icon_sources = match config.favicon {
        Some(source) => get_icons(source).await?,
        None => vec![],
    };
    let mut icons = vec![];
    if !icon_sources.is_empty() {
        icons.push(icon_sources[0].clone());
    }
    
    //styles as one big string
    //let styles_text = get_styles_text(config.styles).await?;
    let styles_text = match config.styles {
        Some(source) => get_styles_text(source).await?,
        None => "".to_string(),
    };

    // scripts as a vec
    //let scripts = get_sources(config.scripts).await?;
    let mut scripts = match config.scripts {
        Some(source) => get_sources(source).await?,
        None => vec![],
    };
    if !scripts.is_empty() {
        // rename wasm_bindgen so that we don't have double definition conflicts
        scripts[0] = scripts[0].replace("wasm_bindgen", "wasm_decoder");
    }

    // binary wasm files
    //let bin = get_wasm(config.wasm)?;
    let bin = match config.wasm {
        Some(source) => get_wasm(source)?,
        None => vec![],
    };

    let markup = html::page(
        styles_text,
        icons,
        scripts,
        bin,
    );


    let html = markup.into_string();
    //println!("html: {}", html);
    html::save_html(html, cli.output)?;

    Ok(())
}

async fn get_icons(
    icon_sources: Vec<AssetSource> 
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut icons: Vec<String> = vec![];
    for icon in icon_sources {
        // we get the path of the file
        // must be local (for now)
        let path_str = match icon {
            AssetSource::Local(path) => path.to_str().ok_or("Invalid favicon path")?.to_string(),
            AssetSource::Remote(_) => return Err("Remote favicons not yet supported".into()),
        };
        // then we get the buffer and encode
        let encoded_icon = crate::encoder::encode_base64(&path_str, "favicon")?;
        icons.push(encoded_icon.text);
    }
    Ok(icons)
}

// append each css file together
async fn get_styles_text(
    style_sources: Vec<AssetSource>
) -> Result<String, Box<dyn Error>> {
    //let styles_text = fetcher::get_css_string(css_urls).await?;
    // init empty string
    let mut styles_text = String::from("");
    for source in style_sources {
        let text = match source {
            AssetSource::Local(path) => fetcher::get_local_file(&path)?,
            AssetSource::Remote(url) => fetcher::get_remote_file(url).await?,
        };
        // append
        styles_text.push_str(&text);
    }
    Ok(styles_text)
}

async fn get_sources(
    sources: Vec<AssetSource>
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut source_text_list: Vec<String> = vec![];
    for source in sources {
        let text = match source {
            AssetSource::Local(path) => fetcher::get_local_file(&path)?,
            AssetSource::Remote(url) => fetcher::get_remote_file(url).await?,
        };
        //append
        source_text_list.push(text);
    }
    Ok(source_text_list)
}

fn get_wasm(
    wasm_modules: Vec<WasmModule> 
) -> Result<Vec<encoder::Base>, Box<dyn Error>> {
    let mut bin: Vec<encoder::Base> = vec![];
    for module in wasm_modules {
        // we get the path of the file
        // must be local (for now)
        let path_str = match module.source{
            AssetSource::Local(path) => path.to_str().ok_or("Invalid WASM path")?.to_string(),
            AssetSource::Remote(_) => return Err("Remote WASM modules not yet supported".into()),
        };
        // then we get the buffer and encode
        let encoded_module = match module.compression {
            CompressionType::Brotli => encoder::encode_brotli_base64(&path_str, &module.id)?,
            CompressionType::None => encoder::encode_base64(&path_str, &module.id)?,
        };
        bin.push(encoded_module);
    }
    Ok(bin)
}


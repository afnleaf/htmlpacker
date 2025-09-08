/*
* packer.rs
*
* the main packing logic
*/

/*
This is the main of the program that packs everything together

we fetch scripts from here
might turn into some type of cli tool that you use with yaml files

local/remote: css, js, text, png, etc
pack it up

compress it with brotli
then encode it in base64
*/


use crate::config::{
    AssetSource, 
    WasmModule, 
    CompressionType, 
    PackerConfig,
};
use crate::cli::{YamlRoot, Cli};
use crate::encoder::{Base};
use crate::encoder;
use crate::wasmbuilder;
use crate::html;
use crate::fetcher;

//
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::{Parser};
use base64::prelude::*;
use sha2::{Sha256, Digest};

// runtime assets on default
const RUNTIME_ICON: &str = include_str!("../core/icon.svg");
const RUNTIME_CORE_JS: &str = include_str!("../core/core.js");
const RUNTIME_DECODER_JS: &str = include_str!("../core/wasm_decoder.js");
const RUNTIME_DECODER_WASM: &[u8] = include_bytes!("../core/wasm_decoder_bg.wasm");

// read yaml from file
// set config from yaml
// need serde structs

// the basic run for API
pub async fn run() -> Result<(), Box<dyn Error>> {
    // parse CLI
    let cli = Cli::parse();
    println!("Config: {}", cli.config.display());
    println!("Output: {}", cli.output.display());
    
    let config = load_config(cli.config).await?;
    pack(config, cli.output).await?;
    Ok(())
}

// loads config from given path, serde yaml->config magic
pub async fn load_config(
    config_path: PathBuf,
) -> Result<PackerConfig, Box<dyn Error>> {
    let yaml_text = fs::read_to_string(config_path)?;
    let yaml_root: YamlRoot = serde_yaml::from_str(&yaml_text)?;
    println!("Extracted yaml");
    //println!("{:?}", yaml_text);
    //println!("{:#?}", &yaml_root.pack);
    let config = crate::cli::set_config_from_yaml(yaml_root.pack).await?;
    println!("Loaded config from yaml");
    Ok(config)
}

// extremely wonky
fn default_runtime(
//    config: &mut PackerConfig,
    icons: &mut Vec<String>,
    scripts: &mut Vec<String>,
    bin: &mut Vec<Base>,
) {
    println!("Default runtime is enabled. \
        Adding icon, core.js and wasm_decoder");

    // favicon
    let encoded_icon_string = 
        BASE64_STANDARD.encode(RUNTIME_ICON.as_bytes());
    icons.insert(0, encoded_icon_string);

    // default scripts
    scripts.push(RUNTIME_DECODER_JS.to_string());
    scripts.push(RUNTIME_CORE_JS.to_string());

    // decoder wasm binary
    let wasm_hash = Sha256::digest(RUNTIME_DECODER_WASM);
    let wasm_hash_string = format!("{:x}", wasm_hash);
    let wasm_encoded_text = BASE64_STANDARD.encode(RUNTIME_DECODER_WASM);
    let decoder_module = Base {
        id: "bin-wasm-decoder".to_string(),
        hash: wasm_hash_string,
        text: wasm_encoded_text,
    };

    bin.push(decoder_module);
}

// have to separate pack from parse cli
// pack takes in a config and an output filename
pub async fn pack(
    config: PackerConfig,
    output: PathBuf,
) -> Result<(), Box<dyn Error>> {
    // make sure to compile our wasm binaries and js glue first
    // how to disable this if already done?
    if let Some(ref modules) = config.wasm {
        wasmbuilder::compile_wasm_modules(modules).await?;
    }

    
    // favicon multiple allowed but forcing only one supported rn
    let icon_sources = match config.favicon {
        Some(source) => get_icons(source).await?,
        None => vec![],
    };
    // this is sort of good but also SLOP
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
    let mut scripts = match config.scripts {
        Some(source) => get_sources(source).await?,
        None => vec![],
    };

    // this is super brittle
    // we don't do it
    // just add wasm_decoder pkg to core
    /*
    if !scripts.is_empty() {
        // rename wasm_bindgen so that we don't have double definition conflicts
        scripts[0] = scripts[0].replace("wasm_bindgen", "wasm_decoder");
    }
    */

    // binary wasm files
    //let bin = get_wasm(config.wasm)?;
    let mut bin = match config.wasm {
        Some(source) => get_wasm(source)?,
        None => vec![],
    };

    // set default runtime for the given configuration
    if config.runtime.enabled {
        default_runtime(
            &mut icons,
            &mut scripts,
            &mut bin,
        );
    }


    let markup = html::page(
        styles_text,
        icons,
        scripts,
        bin,
    );

    let html = markup.into_string();
    //println!("html: {}", html);
    html::save_html(html, output)?;

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
) -> Result<Vec<Base>, Box<dyn Error>> {
    let mut bin: Vec<Base> = vec![];
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




    /*
    let core_icon = vec![
        AssetSource::Local(PathBuf::from(RUNTIME_ICON)),
    ];
    
    for source in core_icon {
        config.favicon.get_or_insert_with(Vec::new).push(source);
    }

    let core_js = vec![
        AssetSource::Local(PathBuf::from(RUNTIME_DECODER_JS)),
        AssetSource::Local(PathBuf::from(RUNTIME_CORE_JS)),
    ];

    for source in core_js {
        config.scripts.get_or_insert_with(Vec::new).push(source);
    }

    let core_wasm = vec![
        WasmModule {
            compile_wasm: false,
            source: AssetSource::Local(PathBuf::from(RUNTIME_DECODER_WASM)),
            id: "bin-wasm-decoder".into(),
            compression: CompressionType::None,
        },
    ];

    for source in core_wasm {
        config.wasm.get_or_insert_with(Vec::new).push(source);
    }
    */

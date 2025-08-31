/*
This is the main of the program that packs everything together

we fetch scripts from here
might turn into some type of cli tool that you use with yaml files

local/remote: css, js, text, png, etc
pack it up

compress it with brotli
then encode it in base64
*/

//
use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use clap::{Parser};
use serde::{Deserialize, Serialize};
use url::Url;

// modules
mod config;
mod encoder;
mod fetcher;
mod html;
mod wasmbuilder;
//use ::htmlpacker::encoder;
//use ::htmlpacker::htmlpacker;
//use ::htmlpacker::wasmbuilder;
//use ::htmlpacker::config::{assetsource, packerconfig, wasmmodule, metaconfig, compressiontype};
//use config::{AssetSource, PackerConfig, WasmModule, MetaConfig, CompressionType};

pub use config::*;
pub use encoder::Base;

// yaml structs
// not sure if this is correct
#[derive(Debug, Serialize, Deserialize)]
struct YamlRoot {
    pack: YamlPack,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlPack {
    meta: Option<YamlMeta>,
    favicon: Option<YamlAssets>,
    css: Option<YamlAssets>,
    scripts: Option<YamlAssets>,
    wasm: Option<HashMap<String, YamlWasmModule>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlMeta {
    title: Option<String>,
    author: Option<String>,
    description: Option<String>,
    keywords: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YamlAssets {
    local: Option<Vec<String>>,
    remote: Option<Vec<String>>,
}


#[derive(Debug, Serialize, Deserialize)]
struct YamlWasmModule {
    path: String,
    id: String,
    #[serde(default = "default_compression")]
    compression: String,
}

fn default_compression() -> String {
    "none".to_string()
}

// clap  
#[derive(Parser)]
#[command(name = "htmlpacker")]
#[command(about = "Pack web assets into a single HTML file")]
struct Cli {
    /// path to the YAML configuration file
    config: PathBuf,
    
    /// output file path (defaults to ./index.html)
    #[arg(short, long, default_value = "./index.html")]
    output: PathBuf,
}


// convert our parsed yaml data into internal config data
async fn set_config_from_yaml(
    pack: YamlPack
) -> Result<PackerConfig, Box<dyn Error>> {
    let mut config = PackerConfig::default();

    config.meta = pack.meta.map(|m| MetaConfig {
        title: m.title,
        author: m.author,
        description: m.description,
        keywords: m.keywords,
    });

    // simple assets conversion
    config.favicon = convert_yaml_assets(pack.favicon)?;
    config.styles = convert_yaml_assets(pack.css)?;
    config.scripts = convert_yaml_assets(pack.scripts)?;

    // wasm modules from hashmap to vec
    config.wasm = pack.wasm.map(|wasm_map| {
        wasm_map.into_iter()
            .map(|(_key, module)| WasmModule {
                id: module.id,
                source: AssetSource::Local(PathBuf::from(module.path)),
                //compression: CompressionType::Brotli, 
                compression: match module.compression.as_str() {
                    "brotli" => CompressionType::Brotli,
                    "none" => CompressionType::None,
                    _ => CompressionType::None,
                }
            })
            .collect()
    });

    Ok(config)
}

// from YamlAsset strings to specific AssetSource
fn convert_yaml_assets(
    assets: Option<YamlAssets>
) -> Result<Option<Vec<AssetSource>>, Box<dyn Error>> {
    match assets {
        None => Ok(None),
        Some(a) => {
            let mut sources = Vec::new();

            // convert local paths
            if let Some(local) = a.local {
                sources.extend(
                    local.into_iter()
                        .map(|path| AssetSource::Local(PathBuf::from(path)))
                );
            }
            
            // convert remote urls
            if let Some(remote) = a.remote {
                for url_str in remote {
                    let url = Url::parse(&url_str)
                        .map_err(|e| format!("Invalid URL '{}': {}", url_str, e))?;
                    sources.push(AssetSource::Remote(url));
                }
            }

            Ok(Some(sources))
        }
    }
}

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
    let config = set_config_from_yaml(yaml_root.pack).await?;
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
        let encoded_icon = encoder::encode_base64(&path_str, "")?;
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


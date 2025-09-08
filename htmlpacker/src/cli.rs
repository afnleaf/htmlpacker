/*
* cli.rs
* 
* where cli commands are parsed
* yaml config declaration
*/

use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;

use clap::{Parser};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::*;

// yaml structs
// not sure if this is correct
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlRoot {
    pub pack: YamlPack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlPack {
    pub runtime: Option<YamlRuntime>,
    pub meta: Option<YamlMeta>,
    pub favicon: Option<YamlAssets>,
    pub css: Option<YamlAssets>,
    pub scripts: Option<YamlAssets>,
    pub wasm: Option<HashMap<String, YamlWasmModule>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlRuntime {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlAssets {
    pub local: Option<Vec<String>>,
    pub remote: Option<Vec<String>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct YamlWasmModule {
    #[serde(default = "default_compile")]
    pub compile_wasm: bool,
    pub id: String,
    pub path: String,
    #[serde(default = "default_compression")]
    pub compression: String,
}

fn default_true() -> bool {
    true
}

fn default_compile() -> bool {
    false
}

fn default_compression() -> String {
    "none".to_string()
}

// clap  
#[derive(Parser)]
#[command(name = "htmlpacker")]
#[command(about = "Pack web assets into a single HTML file")]
pub struct Cli {
    /// path to the YAML configuration file
    pub config: PathBuf,
    
    /// output file path (defaults to ./index.html)
    #[arg(short, long, default_value = "./index.html")]
    pub output: PathBuf,
}


// convert our parsed yaml data into internal config data
pub async fn set_config_from_yaml(
    pack: YamlPack
) -> Result<PackerConfig, Box<dyn Error>> {
    let mut config = PackerConfig::default();

    if let Some(runtime) = pack.runtime {
        config.runtime.enabled = runtime.enabled;
    }

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
                compile_wasm: module.compile_wasm,
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

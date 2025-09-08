/*
* config.rs
*
* where the configuration structs are declared
* also where the config is built
* the config determines how the sources get packed into html
*/

use std::path::PathBuf;
use url::Url;
use serde::{Deserialize, Serialize};

// internal config structs

// enum that distinguishes between local and remote files
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AssetSource {
    Local(PathBuf),
    Remote(Url),
}

impl Default for AssetSource {
    fn default() -> Self {
        AssetSource::Local(PathBuf::new())
    }
}

#[derive(Debug, Deserialize, Serialize)]
//#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    Brotli,
    None,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig { enabled: true }
    }
}

// these are the configuration options for the packer
// this defines the source files that will be packed
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PackerConfig {
    #[serde(default)]
    pub runtime: RuntimeConfig,
    pub meta: Option<MetaConfig>,
    pub favicon: Option<Vec<AssetSource>>,
    pub styles: Option<Vec<AssetSource>>,
    pub scripts: Option<Vec<AssetSource>>,
    pub wasm: Option<Vec<WasmModule>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub enabled: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MetaConfig {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WasmModule {
    pub compile_wasm: bool,
    pub id: String,
    pub source: AssetSource,
    pub compression: CompressionType,
}

/*
* lib.rs
*
* not sure why or what im doing here
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



// modules
mod config;
mod cli;
mod encoder;
mod fetcher;
pub mod packer;
mod html;
mod wasmbuilder;
//use ::htmlpacker::encoder;
//use ::htmlpacker::htmlpacker;
//use ::htmlpacker::wasmbuilder;
//use ::htmlpacker::config::{assetsource, packerconfig, wasmmodule, metaconfig, compressiontype};
//use config::{AssetSource, PackerConfig, WasmModule, MetaConfig, CompressionType};


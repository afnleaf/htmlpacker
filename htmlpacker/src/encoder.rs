/*
The point of this file:
encode the wasm binaries using brotli and base64
return a Base, which means what goes into a <pre> tag inside the html

base64 is highly optimized for encode/decode
we will eventually implement base94 if there is a need

so what do we need to do to encode
we read the file from a filepath

we compress with brotli (or not)

we encode as base64

we return the id and the utf-8 compatible string
*/

use std::fs::File;
use std::io::Read;
use std::error::Error;
use base64::prelude::*;

#[derive(Debug)]
pub struct Base {
    pub id: String,
    pub sl: String,
}

impl Base {
    pub fn new(id: String, sl: String) -> Self {
        Base {
            id: id,
            sl: sl,
        }
    }
}

// get file and read file content
fn get_file_bytes(png_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(png_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// encode any local file to base64
pub fn encode_base64(
    file_path: &str, 
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    let buffer = get_file_bytes(file_path)?;
    let encoded = BASE64_STANDARD.encode(&buffer);
    Ok(Base::new(
        String::from(id),
        encoded
    ))
}

// so we have a buffer of bytes
// lets compress with brotli
// create a buffer for compressed data
pub fn encode_brotli(
    buffer: &Vec<u8>
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut compressed_buffer = Vec::new();
    println!("brotli compression"); 
    brotli::BrotliCompress(
        &mut &buffer[..],          // Input buffer as a Read impl
        &mut compressed_buffer,    // Output buffer as a Write impl
        &brotli::enc::BrotliEncoderParams {
            quality: 11,           // Highest quality (0-11)
            lgwin: 22,             // Window size (recommended 20-22)
            ..Default::default()   // Use defaults for other parameters
        }
    )?;
    println!("brotli done"); 
    Ok(compressed_buffer)
}

// encode both with brotli and then base64
pub fn encode_brotli_base64(
    file_path: &str, 
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    let buffer = get_file_bytes(file_path)?;
    let compressed_buffer = encode_brotli(&buffer)?;
    let encoded = BASE64_STANDARD.encode(&compressed_buffer);
    Ok(Base::new(
        String::from(id),
        encoded
    ))
}


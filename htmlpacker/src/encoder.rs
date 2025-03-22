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

pub fn encode_icon_svg_base64(s: String) -> String {
    BASE64_STANDARD.encode(s.as_bytes())
}

pub fn encode_text_base64(
    s: &str,
    id: &str
) -> Base {
    let encoded = BASE64_STANDARD.encode(s.as_bytes());
    //println!("text: {}", s);
    //println!("b64:  {}", &encoded);
    Base::new(
        String::from(id),
        encoded
    )
}

pub fn encode_png_base64(
    png_path: &str,
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    let buffer = get_file_bytes(png_path)?;
    let encoded = BASE64_STANDARD.encode(&buffer);
    //println!("png size: {}b", buffer.len());
    //println!("b64:  {}\n size:  {}", &encoded, encoded.len());
    Ok(Base::new(
        String::from(id),
        encoded
    ))
}

pub fn encode_wasm_base64(
    wasm_path: &str, 
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    let buffer = get_file_bytes(wasm_path)?;
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

pub fn encode_wasm_base64_brotli(
    wasm_path: &str, 
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    let buffer = get_file_bytes(wasm_path)?;
    let compressed_buffer = encode_brotli(&buffer)?;
    
    // encode the compressed data as base64
    let encoded = BASE64_STANDARD.encode(&compressed_buffer);
    //let encoded = BASE64_STANDARD.encode(&buffer);
    
    // this might be cooked by we do it again
    let comp2 = encode_brotli(&encoded.as_bytes().to_vec())?;
    let encoded2 = BASE64_STANDARD.encode(&comp2);

    Ok(Base::new(
        String::from(id),
        encoded2
    ))

}

/*
pub fn encode_wasm_base94(
    wasm_path: &str,
    id: &str,
) -> Result<Base, Box<dyn Error>> {
    println!("test");
    let mut file = File::open(wasm_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    println!("test2"); 
    // ncode the compressed data as base94
    let encoded = base94::encode(&buffer, 64);
    println!("test3"); 
    Ok(Base::new(
        String::from(id),
        encoded
    ))
}
*/
    //let wasm_bytes = std::fs::read(wasm_path).unwrap();
    //base64::encode(&wasm_bytes)
    /*
    let mut file = File::open(wasm_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    */

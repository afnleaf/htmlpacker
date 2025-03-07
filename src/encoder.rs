use std::fs::File;
use std::io::Read;
use std::error::Error;
use base64::prelude::*;

#[derive(Debug)]
pub struct Base {
    pub id: String,
    pub sl: String,
}

impl Base{
    pub fn new(id: String, sl: String) -> Self {
        Base {
            id: id,
            sl: sl,
        }
    }
}

pub fn encode_text_base64(s: &str) -> Base {
    let encoded = BASE64_STANDARD.encode(s.as_bytes());

    //println!("text: {}", s);
    //println!("b64:  {}", &encoded);

    Base::new(
        String::from("bin-text"),
        encoded
    )
}

pub fn encode_png_base64(png_path: &str) -> Result<Base, Box<dyn Error>> {

    let mut file = File::open(png_path)?;
    // read file content
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let encoded = BASE64_STANDARD.encode(&buffer);

    //println!("png size: {}b", buffer.len());
    //println!("b64:  {}\n size:  {}", &encoded, encoded.len());

    Ok(Base::new(
        String::from("bin-png"),
        encoded
    ))
}

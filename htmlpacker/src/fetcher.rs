/*
* fetcher.rs
*
* fetches source files whether they are local or external
*/

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path};
use std::error::Error;
// crates
use url::Url;


// go through each external script file
// saving 
pub async fn get_external_scripts_text(
    script_urls: Vec<&str>
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut script_strings: Vec<String> = vec![];
    for url in script_urls {
        script_strings.push(reqwest::get(url).await?.text().await?);
    }
    Ok(script_strings) 
}

// go through each local script file
pub fn get_local_scripts_text(
    script_path_literals: Vec<&str>
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut script_strings: Vec<String> = vec![];
    for path in script_path_literals {
        script_strings.push(
            get_local_script(Path::new(path)).unwrap()
        )
    }
    Ok(script_strings)
}

pub fn get_local_script(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

pub fn get_local_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

pub async fn get_remote_file(url: Url) -> Result<String, Box<dyn Error>> {
    let text = reqwest::get(url).await?.text().await?;
    Ok(text)
}

// append each external css file together
pub async fn get_css_string(
    css_urls: Vec<&str>
) -> Result<String, Box<dyn Error>> {
    let mut css_string = String::from(""); 
    for url in css_urls {
        css_string += &reqwest::get(url).await?.text().await?;
    }
    Ok(css_string)
}

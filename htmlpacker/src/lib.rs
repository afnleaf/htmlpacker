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
use std::path::PathBuf;
use url::Url;
use std::error::Error;

// modules
pub mod encoder;
pub mod htmlpacker;
pub mod wasmbuilder;
//use ::htmlpacker::encoder;
//use ::htmlpacker::htmlpacker;
//use ::htmlpacker::wasmbuilder;
//


// enum that distinguishes between local and remote files
#[derive(Debug)]
pub enum AssetSource {
    Local(PathBuf),
    Remote(Url),
}

impl Default for AssetSource {
    fn default() -> Self {
        AssetSource::Local(PathBuf::new())
    }
}

#[derive(Debug)]
pub enum CompressionType {
    Brotli,
    None,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::None
    }
}

// these are the configuration options for the packer
// this defines the source files that will be packed
#[derive(Debug, Default)]
pub struct PackerConfig {
    pub meta: Option<MetaConfig>,
    pub favicon: Option<Vec<AssetSource>>,
    pub styles: Option<Vec<AssetSource>>,
    pub scripts: Option<Vec<AssetSource>>,
    pub wasm: Option<Vec<WasmModule>>,
}

#[derive(Debug, Default)]
pub struct MetaConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
}

#[derive(Debug, Default)]
pub struct WasmModule {
    pub id: String,
    pub source: AssetSource,
    pub compression: CompressionType,
}

// hardcoded config for testing of config logic
// this is so ugly
async fn set_config_hard() -> Result<PackerConfig, Box<dyn Error>> {
    let mut config = PackerConfig::default();
    
    // HTML Metadata
    // should favicon be part of this?
    config.meta = Some(MetaConfig {
        title: Some("htmlpacker".to_string()),
        // we could add a system time with chrono for when it was packed (default)
        description: Some("packed by htmlpacker".to_string()),
        ..Default::default()
    });

    config.favicon = Some(vec![AssetSource::Local(PathBuf::from("../public/icon.svg"))]);

    // let normalize_css = Url::parse("https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css")?;
    // config.styles = Some(vec![AssetSource::Remote(normalize_css)]);
    config.styles = Some(vec![]);

    let mut scripts = Vec::new();
    scripts.push(AssetSource::Local(PathBuf::from("../wasm_decoder/pkg/wasm_decoder.js",)));
    scripts.push(AssetSource::Local(PathBuf::from("../wasm_modules/pkg/wasm_modules.js",)));
    scripts.push(AssetSource::Local(PathBuf::from("../public/decoder.js")));
    scripts.push(AssetSource::Local(PathBuf::from("../public/app2.js")));
    config.scripts = Some(scripts);

    let mut wasm_modules = Vec::new();
    wasm_modules.push(WasmModule {
        id: "bin-wasm-decoder".to_string(),
        source: AssetSource::Local(PathBuf::from("../wasm_decoder/pkg/wasm_decoder_bg.wasm",)),
        compression: CompressionType::None,
    });
    wasm_modules.push(WasmModule {
        id: "bin-wasm".to_string(),
        source: AssetSource::Local(PathBuf::from("../wasm_modules/pkg/wasm_modules_bg.wasm",)),
        compression: CompressionType::Brotli,
    });
    config.wasm = Some(wasm_modules);

    Ok(config)
}

pub async fn pack() -> Result<(), Box<dyn Error>> {
    // make sure to compile our wasm binaries and js glue first
    wasmbuilder::compile_wasm_modules().await?;
    
    // set our hardcoded conf (later we parse from yaml)
    let config = set_config_hard().await?;

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

    let markup = htmlpacker::page(
        styles_text,
        icons,
        scripts,
        bin,
    );

    let html = markup.into_string();
    htmlpacker::save_html(html)?;

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
    //let styles_text = htmlpacker::get_css_string(css_urls).await?;
    // init empty string
    let mut styles_text = String::from("");
    for source in style_sources {
        let text = match source {
            AssetSource::Local(path) => htmlpacker::get_local_file(&path)?,
            AssetSource::Remote(url) => htmlpacker::get_remote_file(url).await?,
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
            AssetSource::Local(path) => htmlpacker::get_local_file(&path)?,
            AssetSource::Remote(url) => htmlpacker::get_remote_file(url).await?,
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

/*
pub async fn pack1() -> Result<(), Box<dyn Error>> {
    // make sure to compile our wasm binaries and js glue first
    wasmbuilder::compile_wasm_modules().await?;
    // there must be a way to refactor this build process.
    // configuration of build file
    //
    // obviously handle the wasm-pack first
    let css_urls = vec![
        //"https://cdnjs.cloudflare.com/ajax/libs/normalize/8.0.1/normalize.min.css"
    ];

    // if you want to use javascript libraries,
    // do we need to implement modules tag mode?
    let external_scripts_list = vec![
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/build/three.min.js",
        //"https://cdn.jsdelivr.net/npm/three@0.132.2/examples/js/controls/TrackballControls.min.js"
    ];

    let local_scripts_list = vec![
        "../wasm_decoder/pkg/wasm_decoder.js",
        "../wasm_modules/pkg/wasm_modules.js", // bindgen
        "../public/decoder.js", // decode wasm from base64
        "../public/app2.js", // app logic
        //"../public/app.js", // app logic
        //"../public/script.js", // test script
    ];

    // external css
    let css_text = htmlpacker::get_css_string(css_urls).await?;

    // favicon processing
    //let icon_path = "../public/icon.svg";
    //let icon_svg = htmlpacker::get_local_script(Path::new(icon_path)).unwrap();
    //let icon_svg64 = encoder::encode_icon_svg_base64(icon_svg);
    let icon_svg64 = encoder::encode_base64("../public/icon.svg", "").unwrap();

    //println!("{:?}", icon_svg64);
    let icons = vec![icon_svg64.text];

    // external scripts to fetch
    let external_scripts_text = htmlpacker::get_external_scripts_text(
        external_scripts_list).await?;

    // get local scripts
    let mut local_scripts_text = htmlpacker::get_local_scripts_text(
        local_scripts_list)?;
    //println!("{:?}", external_scripts_text);
    //println!("{:?}", local_scripts_text);

    // rename wasm_bindgen so that we don't have double definition conflicts
    local_scripts_text[0] = local_scripts_text[0].replace(
        "wasm_bindgen",
        "wasm_decoder");

    // base64 encoder
    // encode_base64 is all you need now
    //let text64 = encoder::encode_text_base64("hello world!!!\nt. packer", "bin-text");
    //let png64 = encoder::encode_png_base64("../public/wizard.png", "bin-png")?;
    //let wasm64 = encoder::encode_wasm_base94("../wasm_modules/pkg/wasm_modules_bg.wasm", "bin-wasm")?;
    // this decoder is the reason we can do brotli decompress
    let wasm_decoder64 = encoder::encode_base64(
        "../wasm_decoder/pkg/wasm_decoder_bg.wasm", 
        "bin-wasm-decoder")?;
    let wasm_module64 = encoder::encode_brotli_base64(
        "../wasm_modules/pkg/wasm_modules_bg.wasm", 
        "bin-wasm")?;
    
    let bin: Vec<encoder::Base> = vec![
        //text64,
        //png64,
        wasm_decoder64,
        wasm_module64,
        //texture64,
        //model64,
    ];

    let markup = htmlpacker::page(
        css_text,
        icons,
        external_scripts_text,
        local_scripts_text,
        bin,
    );

    let html = markup.into_string();
    //println!("{}", html);
    htmlpacker::save_html(html)?;

    Ok(())
}
*/

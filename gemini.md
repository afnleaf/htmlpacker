Of course. Transforming this collection of code from a command-line application script into a well-structured, reusable library API involves several key steps. The goal is to separate the core logic (the "what") from the application-specific implementation (the "how," e.g., command-line parsing, reading a specific YAML file).

Here is a step-by-step guide on how to refactor your code into a proper library and a binary that uses it.

### 1. The Core Principles of a Library API

1.  **Clear Entry Point**: A consumer of your library should have one or a few main functions to call.
2.  **Well-Defined Data Structures**: The library should operate on well-defined structs for configuration, not on raw file paths or YAML strings. The `PackerConfig` you've started is perfect for this.
3.  **Robust Error Handling**: Instead of `Box<dyn Error>`, a library should expose a custom error `enum`. This allows the consumer to programmatically handle different kinds of errors.
4.  **Separation of Concerns**: The library should not handle CLI parsing (`clap`), nor should it build WASM modules as a side effect of packing. These are concerns for the application *using* the library.
5.  **No Side Effects (Where Possible)**: A library function like `pack` should ideally not print to the console (`println!`). It should return data or errors. Logging can be handled with a crate like `log`.

---

### 2. Proposed Project Structure

This is the standard way to structure a Rust project that has both a library and a binary executable.

```
htmlpacker/
├── Cargo.toml
├── src/
│   ├── lib.rs          # The library's main file (API)
│   ├── main.rs         # The binary executable's main file (CLI)
│   │
│   ├── config.rs       # Module for configuration structs
│   ├── encoder.rs      # Module for Base64/Brotli encoding
│   ├── error.rs        # Module for the custom library error type
│   ├── fetch.rs        # Module for fetching local/remote assets
│   ├── html.rs         # Module for HTML generation (maud)
│   └── wasm.rs         # Module for the WASM compilation logic
│
└── examples/
    └── basic_usage.rs  # Example of how to use the library programmatically
```

---

### 3. Refactoring Step-by-Step

#### Step 1: Create a Custom Error Type (`src/error.rs`)

This makes your library much more professional.

```rust
// src/error.rs
use std::io;
use thiserror::Error; // Add `thiserror = "1.0"` to Cargo.toml

#[derive(Error, Debug)]
pub enum PackerError {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Invalid asset path: {0}")]
    InvalidPath(String),

    #[error("Brotli compression failed: {0}")]
    Compression(String),

    #[error("WASM build command failed: {0}")]
    WasmBuild(String),

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}
```

#### Step 2: Organize Code into Modules

Move the relevant code from your original file into the new module files.

**`src/config.rs`**: All your configuration structs. They are the public "data contract" for your library.
```rust
// src/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

// Make all structs and fields public so they can be used by the library consumer
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AssetSource {
    Local(PathBuf),
    Remote(Url),
}
// ... (rest of the config structs: PackerConfig, WasmModule, etc.)
// ... make sure all fields and structs intended for public use are `pub`
```

**`src/encoder.rs`**: Your encoding logic. Remove the `println!` statements.
```rust
// src/encoder.rs
use super::error::PackerError; // Use our custom error
// ... other imports ...

#[derive(Debug, Clone)] // Add Clone
pub struct Base {
    pub id: String,
    pub hash: String,
    pub text: String,
}
// ... impl Base ...

// The public API for encoding.
pub fn encode_base64_from_path(file_path: &str, id: &str) -> Result<Base, PackerError> {
    let buffer = get_file_bytes(file_path)?;
    // ... logic ...
    Ok(Base::new(/*...*/))
}

pub fn encode_brotli_base64_from_path(file_path: &str, id: &str) -> Result<Base, PackerError> {
    let buffer = get_file_bytes(file_path)?;
    let compressed = encode_brotli(&buffer)?;
    // ... logic ...
    Ok(Base::new(/*...*/))
}

// Helper function changed to return PackerError
fn get_file_bytes(file_path: &str) -> Result<Vec<u8>, PackerError> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// This function now returns a proper error
fn encode_brotli(buffer: &[u8]) -> Result<Vec<u8>, PackerError> {
    let mut compressed_buffer = Vec::new();
    let params = brotli::enc::BrotliEncoderParams { quality: 9, lgwin: 22, ..Default::default() };
    brotli::BrotliCompress(&mut &buffer[..], &mut compressed_buffer, &params)
        .map_err(|e| PackerError::Compression(e.to_string()))?;
    Ok(compressed_buffer)
}
```

**`src/html.rs`**: Your `maud` template logic. These can be kept mostly as-is, but they'll be internal to the library.
```rust
// src/html.rs
use maud::{html, Markup, PreEscaped, DOCTYPE};
use crate::encoder::Base; // Use crate-relative path

// This is the only function that needs to be public for this module.
pub fn generate_html(
    css: String,
    icons: Vec<String>,
    js: Vec<String>,
    bin: Vec<Base>,
) -> Markup {
    page(css, icons, js, bin)
}

// Keep the rest of the functions (page, head, scripts, etc.) private to this module.
fn page(...) -> Markup { /* ... */ }
fn head(...) -> Markup { /* ... */ }
// ... etc ...
```

**`src/fetch.rs`**: Logic for getting asset content.
```rust
// src/fetch.rs
use crate::config::AssetSource;
use crate::error::PackerError;
// ... other imports ...

pub async fn get_asset_string(source: &AssetSource) -> Result<String, PackerError> {
    match source {
        AssetSource::Local(path) => Ok(tokio::fs::read_to_string(path).await?),
        AssetSource::Remote(url) => Ok(reqwest::get(url.clone()).await?.text().await?),
    }
}

pub async fn get_all_assets_as_string(sources: &[AssetSource]) -> Result<String, PackerError> {
    let mut combined = String::new();
    for source in sources {
        combined.push_str(&get_asset_string(source).await?);
        combined.push('\n');
    }
    Ok(combined)
}

pub async fn get_all_assets_as_vec(sources: &[AssetSource]) -> Result<Vec<String>, PackerError> {
    let mut results = Vec::new();
    for source in sources {
        results.push(get_asset_string(source).await?);
    }
    Ok(results)
}
```

**`src/wasm.rs`**: The isolated WASM compilation logic.
```rust
// src/wasm.rs
use crate::error::PackerError;
use std::process::Command;
use tokio::task;

/// Compiles a WASM module in a given directory using `wasm-pack`.
/// This is a separate utility function exposed by the library.
pub async fn build_wasm(dir: &str) -> Result<(), PackerError> {
    let dir_owned = dir.to_string();
    let status = task::spawn_blocking(move || {
        Command::new("wasm-pack")
            .current_dir(&dir_owned)
            .args(["build", "--target", "no-modules"])
            .status()
    })
    .await??;

    if !status.success() {
        return Err(PackerError::WasmBuild(format!(
            "Failed to compile WASM in {}",
            dir
        )));
    }
    Ok(())
}
```

#### Step 3: Define the Library API in `src/lib.rs`

This is the public-facing part of your library.

```rust
// src/lib.rs

// 1. Declare modules
mod config;
mod encoder;
mod error;
mod fetch;
mod html;
pub mod wasm;

// 2. Expose the public API parts
pub use config::{AssetSource, CompressionType, PackerConfig, WasmModule};
pub use error::PackerError;
use encoder::Base;

/// The main entry point for the htmlpacker library.
///
/// Takes a configuration struct and returns a single HTML string.
/// This function is async because it may need to fetch assets from the network.
pub async fn pack(config: &PackerConfig) -> Result<String, PackerError> {
    // 1. Fetch and combine CSS styles
    let styles = match &config.styles {
        Some(sources) => fetch::get_all_assets_as_string(sources).await?,
        None => String::new(),
    };

    // 2. Fetch and encode favicons
    let mut icons = Vec::new();
    if let Some(sources) = &config.favicon {
        for source in sources {
            if let AssetSource::Local(path) = source {
                let path_str = path.to_str().ok_or_else(|| {
                    PackerError::InvalidPath(format!("Invalid favicon path: {:?}", path))
                })?;
                let encoded_icon = encoder::encode_base64_from_path(path_str, "")?;
                icons.push(encoded_icon.text);
            }
            // Add remote favicon support if needed
        }
    }

    // 3. Fetch scripts
    let mut scripts = match &config.scripts {
        Some(sources) => fetch::get_all_assets_as_vec(sources).await?,
        None => Vec::new(),
    };
    // Your special logic for wasm_bindgen glue code
    if !scripts.is_empty() {
        scripts[0] = scripts[0].replace("wasm_bindgen", "wasm_decoder");
    }

    // 4. Encode WASM binaries
    let mut bin: Vec<Base> = Vec::new();
    if let Some(modules) = &config.wasm {
        for module in modules {
            if let AssetSource::Local(path) = &module.source {
                let path_str = path.to_str().ok_or_else(|| {
                    PackerError::InvalidPath(format!("Invalid WASM path: {:?}", path))
                })?;
                let encoded_module = match module.compression {
                    CompressionType::Brotli => {
                        encoder::encode_brotli_base64_from_path(path_str, &module.id)?
                    }
                    CompressionType::None => encoder::encode_base64_from_path(path_str, &module.id)?,
                };
                bin.push(encoded_module);
            }
        }
    }
    
    // 5. Generate the final HTML
    let markup = html::generate_html(styles, icons, scripts, bin);
    
    Ok(markup.into_string())
}
```

#### Step 4: Create the Binary in `src/main.rs`

The binary is now just a *consumer* of your library. Its only job is to handle CLI arguments, read the config file, and call the library.

```rust
// src/main.rs
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use serde_yaml::from_str as from_yaml;

// Use our own library
use htmlpacker::{pack, wasm, PackerConfig, PackerError};

// The YAML parsing structs can be kept private to the binary
// or moved to a separate module within the binary if they get complex.
// ... (Your YamlRoot, YamlPack, etc. structs go here) ...
// ... (The `set_config_from_yaml` conversion function goes here) ...

#[derive(Parser)]
#[command(name = "htmlpacker")]
struct Cli {
    #[arg(short, long)]
    config: PathBuf,
    
    #[arg(short, long, default_value = "./packed.html")]
    output: PathBuf,

    #[arg(long)]
    skip_wasm_build: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), PackerError> {
    let cli = Cli::parse();
    
    // 1. Optionally build WASM
    if !cli.skip_wasm_build {
        println!("Building WASM modules...");
        // This is a good place to add more complex logic, like finding
        // wasm project directories from the config. For now, hardcoded.
        wasm::build_wasm("../wasm_decoder").await?;
        wasm::build_wasm("../wasm_modules").await?;
        println!("WASM build complete.");
    }

    // 2. Read and parse config
    println!("Reading config from: {}", cli.config.display());
    let yaml_text = fs::read_to_string(&cli.config)?;
    // A simplified conversion, assuming your YAML structs are available
    let packer_config: PackerConfig = from_yaml(&yaml_text)
        .map_err(|e| PackerError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    
    // 3. Call the library's main function
    println!("Packing assets...");
    let html_output = pack(&packer_config).await?;
    
    // 4. Write the output
    println!("Writing output to: {}", cli.output.display());
    fs::write(&cli.output, html_output)?;
    
    println!("Packing successful!");
    Ok(())
}
```

#### Step 5: Update `Cargo.toml`

Finally, tell Cargo that your package has both a library and a binary.

```toml
[package]
name = "htmlpacker"
version = "0.1.0"
edition = "2021"

# This section is crucial
[lib]
name = "htmlpacker"
path = "src/lib.rs"

[[bin]]
name = "htmlpacker"
path = "src/main.rs"

[dependencies]
base64 = "0.21"
brotli = "3.3"
clap = { version = "4.4", features = ["derive"] }
maud = "0.25"
reqwest = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
sha2 = "0.10"
thiserror = "1.0" # New dependency
tokio = { version = "1", features = ["full"] }
url = { version = "2.4", features = ["serde"] }
```

### Summary of Benefits

1.  **Reusability**: Anyone (including you in other projects) can now add `htmlpacker` as a dependency and use `htmlpacker::pack()` programmatically, without needing a CLI or YAML files.
2.  **Testability**: You can now write unit tests for your library's functions (`pack`, `encoder` functions, etc.) by creating `PackerConfig` structs in your test code. You couldn't easily test your old `main` function.
3.  **Clearer API**: The public API is explicitly defined in `src/lib.rs` with structs from `src/config.rs` and the error type from `src/error.rs`. It's obvious how to use your code.
4.  **Separation of Concerns**: The binary (`main.rs`) is now a thin wrapper. All the heavy lifting and logic is in the library. If you wanted to create a GUI for your packer, it would also just be another consumer of the library.

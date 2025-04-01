use std::error::Error;
use std::process::Command;
use tokio::task;

// 
pub async fn compile_wasm_modules() -> Result<(), Box<dyn Error>> {
    //build_wasm("../wasm_decoder").unwrap();
    //build_wasm("../wasm_modules").unwrap();
    let decoder_build = build_wasm("../wasm_decoder");
    let modules_build = build_wasm("../wasm_modules");
    
    // Join both futures and get their results
    let (decoder_result, modules_result) = tokio::join!(decoder_build, modules_build);
    
    // Check results - exit with error code if any build failed
    if let Err(err) = decoder_result {
        eprintln!("Decoder build failed: {}", err);
        std::process::exit(1);
    }
    
    if let Err(err) = modules_result {
        eprintln!("Modules build failed: {}", err);
        std::process::exit(1);
    }
    
    // If we get here, both builds succeeded
    println!("All WASM builds completed successfully!");
    Ok(())
}

// build script for our wasm modules
//.env("RUSTFLAGS", "--cfg getrandom_backend=\"wasm_js\"")
// this line is for when we add random
// wasm-pack build --target no-modules
async fn build_wasm(
    dir: &str
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Building WASM in {}", dir);

    // spawn command in blocking task
    let dir_owned = dir.to_string();
    let status = task::spawn_blocking(move || {
        Command::new("wasm-pack")
            .current_dir(&dir_owned)
            .args(&[
                "build",
                "--target",
                "no-modules",
            ])
            .status()
    })
    .await?;

    if !status?.success() {
        println!("HELO THERE!");
        return Err(format!("Failed to compiled WASM in {}", dir).into());
    }

    println!("WASM compiled in {}.", dir);
    Ok(())
}


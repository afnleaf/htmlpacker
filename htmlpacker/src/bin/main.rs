/*
* tiny bin entry point 
*/

use htmlpacker::*;

#[tokio::main]
async fn main() {

    if let Err(e) = htmlpacker::packer::pack().await {
        eprintln!("\nError: Failed to pack assets: {}", e);

        std::process::exit(1);
    }
    //let _ = pack().await;
}


/*
* tiny bin entry point 
*/

use htmlpacker::pack;

#[tokio::main]
async fn main() {

    if let Err(e) = pack().await {
        eprintln!("\nError: Failed to pack assets: {}", e);

        std::process::exit(1);
    }
    //let _ = pack().await;
}


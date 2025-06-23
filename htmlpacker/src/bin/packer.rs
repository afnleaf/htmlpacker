/*
* tiny bin entry point 
*/

use htmlpacker::pack;

#[tokio::main]
async fn main() {
    let _ = pack().await;
}


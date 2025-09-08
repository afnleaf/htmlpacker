/*
* main.rs
*
* tiny bin entry point
*/

use std::error::Error;
use htmlpacker::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    packer::run().await
}


use std::error::Error;
mod index;
mod configs;
mod elastic_hyperion;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    index::start_index_block_result_v0().await;
    Ok(())
}
pub mod crawlers;

use crawlers::{asia_aliance_parser, asaka_parser};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    asia_aliance_parser().await?;
    asaka_parser().await?;

    Ok(())
}

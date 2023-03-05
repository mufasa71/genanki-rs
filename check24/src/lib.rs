pub mod crawlers;

use crawlers::{asia_alliance_parser, asaka_parser};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    asia_alliance_parser(None).await?;
    asaka_parser(None).await?;

    Ok(())
}

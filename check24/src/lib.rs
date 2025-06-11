pub mod crawlers;

use crawlers::{asaka_parser, asia_alliance_parser};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    asia_alliance_parser(None).await?;
    asaka_parser(None).await?;

    Ok(())
}

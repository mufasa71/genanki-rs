mod simple_logger;

use log::{error, info, warn, SetLoggerError};
use simple_logger::SimpleLogger;
use std::io::Cursor;
use std::path::Path;
use tokio::task::JoinSet;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init().unwrap();
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let body: serde_json::Value = {
        client
            .get("https://www.reddit.com/r/wallpaper/top.json")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?
    };

    let mut set = JoinSet::new();

    let children = body["data"]["children"].as_array();

    if let Some(children) = children {
        for child in children {
            let url = child["data"]["url"].as_str();
            if url.is_none() {
                continue;
            }
            let url_unwrap = String::from(url.unwrap());
            let title = child["data"]["title"].as_str().unwrap();
            let filename = format!("{}/Pictures/{}.jpg", env!("HOME"), title);
            let is_file_exists = Path::is_file(Path::new(&filename));
            info!("Download {}, {} ", url_unwrap, title);
            if is_file_exists {
                warn!("File exists! {}", filename);
                continue;
            }
            set.spawn(async move { download_image(url_unwrap, filename).await });
        }
    }

    while let Some(r) = set.join_next().await {
        match r {
            Ok(_) => {
                println!("Done!");
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn download_image(url: String, filename: String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let image = client.get(url).send().await?;

    if image.status() == reqwest::StatusCode::OK {
        let mut file = std::fs::File::create(filename.clone()).unwrap();
        let content_bytes = image.bytes().await?;
        let mut content = Cursor::new(content_bytes);
        std::io::copy(&mut content, &mut file).unwrap();

        info!("Image saved to {}\n", filename);
    } else {
        error!("Error: ${:?}, Response status {}", filename, image.status());
    }

    Ok(())
}

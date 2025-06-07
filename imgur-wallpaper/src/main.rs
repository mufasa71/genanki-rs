mod cli;
mod data;

use clap::Parser;
use cli::Cli;
use data::{DownloadImageError, GetFileNameError, RedditData, Wallpaper};
use log::{Level, error, info, warn};
use std::error::Error;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let output = cli.output.as_deref().unwrap_or("Pictures/Wallpapers");
    let level = match cli.debug {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => Level::Error,
    };
    simple_logger::init_with_level(level).expect("Logger init failed");
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    let mut q: Vec<(&str, String)> = vec![];

    if let Some(limit) = cli.limit {
        q.push(("limit", limit.to_string()));
    }

    if let Some(t) = cli.t {
        q.push(("t", t.to_string()))
    }

    let body = {
        client
            .get("https://www.reddit.com/r/wallpaper/top.json")
            .query(&q)
            .send()
            .await?
            .json::<RedditData>()
            .await?
    };

    let mut set = JoinSet::new();
    let children = body.data.children;

    for child in children {
        let Wallpaper { url, id } = child.data;
        let file_name = get_file_name(&url, &id);

        match file_name {
            Err(e) => error!("Cannot get file name: {:?}", e),
            Ok(file_name) => {
                let mut file_path = dirs::home_dir().expect("Home dir not found!");
                file_path = file_path.join(output).join(file_name);

                let is_file_exists = Path::is_file(file_path.as_path());

                if is_file_exists {
                    warn!("File {:?} exists - skipping", file_path);
                } else {
                    set.spawn(async move { download_image(&url, &file_path).await });
                }
            }
        }
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(result) => match result {
                Ok(url) => info!("Download finished for {}", url),
                Err(error) => {
                    error!("Error occured: {:?}", error);
                }
            },
            Err(e) => {
                error!("Error spawning process: {:?}", e);
            }
        }
    }

    Ok(())
}

fn get_file_name(url: &str, id: &str) -> Result<String, GetFileNameError> {
    let ext = image::ImageFormat::from_path(url);
    match ext {
        Ok(f) => match f {
            image::ImageFormat::Png => Ok(format!("wallpapers-{}.png", id)),
            image::ImageFormat::Jpeg => Ok(format!("wallpapers-{}.jpg", id)),
            _ => Err(GetFileNameError {
                message: format!("Not supported file extension: {:?}", f),
            }),
        },
        Err(_) => Err(GetFileNameError {
            message: format!("File extension not determined for url: {:?}", url),
        }),
    }
}

async fn download_image(
    url: &str,
    file_path_buf: &PathBuf,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Download from {} to {:?}", url, file_path_buf);

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let image = client.get(url).send().await?;

    if image.status() == reqwest::StatusCode::OK {
        let mut file = std::fs::File::create(file_path_buf)?;
        let content_bytes = image.bytes().await?;
        let mut content = Cursor::new(content_bytes);
        std::io::copy(&mut content, &mut file)?;

        // check if file is image by opening it
        let image = image::open(file_path_buf);

        match image {
            Ok(_) => {}
            Err(error) => {
                std::fs::remove_file(file_path_buf)?;

                if let Some(error) = error.source() {
                    return Err(Box::new(DownloadImageError {
                        message: error.to_string(),
                    }));
                }
            }
        }

        info!("Image saved in {:?}", file_path_buf);
        Ok(String::from(url))
    } else {
        Err(Box::new(DownloadImageError {
            message: String::from("Image download error"),
        }))
    }
}

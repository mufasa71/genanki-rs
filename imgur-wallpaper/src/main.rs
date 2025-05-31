mod data;

use data::{DownloadImageError, GetFileNameError, RedditData, Wallpaper};
use log::{error, info, warn};
use std::error::Error;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::init_with_env().expect("Logger init failed");
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let body = {
        client
            .get("https://www.reddit.com/r/wallpaper/top.json")
            .send()
            .await?
            .json::<RedditData>()
            .await?
    };

    let mut set = JoinSet::new();
    let children = body.data.children;

    for child in children {
        let Wallpaper { url, title } = child.data;
        let file_name = get_file_name(&url, &title);

        match file_name {
            Err(e) => error!("Cannot get file name: {:?}", e),
            Ok(file_name) => {
                let mut file_path = PathBuf::from(env!("HOME"));
                file_path = file_path.join("Pictures").join(file_name);

                let is_file_exists = Path::is_file(file_path.as_path());

                if is_file_exists {
                    warn!("File {:?} exists - skipping", file_path);
                } else {
                    info!("Download from {} to {:?}", url, file_path);

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

fn get_file_name(url: &str, title: &str) -> Result<String, GetFileNameError> {
    let ext = image::ImageFormat::from_path(url);
    let replace_chars_regex =
        regex::Regex::new(r"[^a-zA-Z0-9_\-.]").expect("Regex has invalid pattern");
    let title = &replace_chars_regex.replace_all(title, "_").to_string();
    match ext {
        Ok(f) => match f {
            image::ImageFormat::Png => Ok(format!("{}.png", title)),
            image::ImageFormat::Jpeg => Ok(format!("{}.jpg", title)),
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

        info!("Image saved to {:?}", file_path_buf);
    } else {
        error!(
            "Error: ${:?}, Response status {}",
            file_path_buf,
            image.status()
        );
    }

    Ok(String::from(url))
}

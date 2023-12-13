use std::io::Cursor;
use std::path::Path;
use tokio::task::JoinSet;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            println!("Download url = {:?}, title = {:?}", url, title);
            if is_file_exists {
                println!("File exists!");
                continue;
            }
            set.spawn(async move { download_image(url_unwrap, filename).await });
        }
    }

    while let Some(_) = set.join_next().await {
        println!("Downloaded image");
    }

    Ok(())
}

async fn download_image(
    url: String,
    filename: String,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build();
    if client.is_err() {
        return Err(Box::new(client.err().unwrap()));
    }
    let image = client.unwrap().get(url).send().await;
    if image.is_err() {
        return Err(Box::new(image.err().unwrap()));
    }
    let file = std::fs::File::create(filename);
    if file.is_err() {
        return Err(Box::new(file.err().unwrap()));
    }
    let mut file = file.unwrap();
    let content_bytes = image.unwrap().bytes().await;
    if content_bytes.is_err() {
        return Err(Box::new(content_bytes.err().unwrap()));
    }
    let mut content = Cursor::new(content_bytes.unwrap());

    let copy_result = std::io::copy(&mut content, &mut file);

    if copy_result.is_err() {
        return Err(Box::new(copy_result.err().unwrap()));
    }
    Ok(())
}

use std::io::Cursor;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let body = client
        .get("https://www.reddit.com/r/wallpaper/top.json")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let url = body["data"]["children"][0]["data"]["preview"]["images"][0]["source"]["url"]
        .as_str()
        .unwrap()
        .replace("amp;", "");

    let title = body["data"]["children"][0]["data"]["title"]
        .as_str()
        .unwrap();
    let filename = format!("{}/Pictures/{}.jpg", env!("HOME"), title);
    let is_file_exists = Path::is_file(Path::new(&filename));

    println!("Download url = {:?}, title = {:?}", url, title);

    if is_file_exists {
        println!("File exists!");
        return Ok(());
    }
    let image = client.get(url).send().await?;
    let mut file = std::fs::File::create(filename)?;
    let mut content = Cursor::new(image.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

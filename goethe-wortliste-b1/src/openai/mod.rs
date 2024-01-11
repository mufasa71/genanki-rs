use serde_json::json;

const OPENAI_API_KEY: &str = env!("OPENAI_API_KEY");

pub async fn image(text: &str) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let params_json = json!(
  {
    "prompt": text,
    "n": 1,
    "size": "1024x1024",
    "model": "dall-e-3"
  }

    );
  let answer = client
    .post("https://api.openai.com/v1/images/generations")
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {}", OPENAI_API_KEY))
    .json(&params_json)
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;

  println!("answer: {:?}", answer);
  Ok("".to_string())
}

const DEEPL_AUTH_KEY: &str = env!("DEEPL_AUTH_KEY");

pub async fn deep_translate(text: &str) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let params = [
    ("text", text),
    ("target_lang", "RU"),
    ("source_lang", "DE"),
    ("context", "Goethe Zertifikat B1 Wortliste"),
  ];
  let answer = client
    .post("https://api-free.deepl.com/v2/translate")
    .header(
      "Authorization",
      format!("DeepL-Auth-Key {}", DEEPL_AUTH_KEY),
    )
    .form(&params)
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;

  Ok(
    answer["translations"][0]["text"]
      .as_str()
      .unwrap()
      .to_string(),
  )
}

const DEEPL_AUTH_KEY: &str = env!("DEEPL_AUTH_KEY");

pub async fn translate_ai() -> Result<(), Box<dyn std::error::Error>> {
  let text = format!("{}", "Hello welt!");

  let client = reqwest::Client::new();
  let params = [
    ("text", text.as_str()),
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

  println!("text: {:?}, answer: {}", text, answer);

  Ok(())
}

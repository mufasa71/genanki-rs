use crate::db;

const DEEPL_AUTH_KEY: &str = env!("DEEPL_AUTH_KEY");

async fn translate(text: &str, context: &str) -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let params = [
    ("text", text),
    ("context", context),
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
  let translation = answer["translations"][0]["text"]
    .as_str()
    .unwrap()
    .to_string();

  Ok(translation)
}

pub async fn deep_translate(
  word_item: &db::WordItem,
) -> Result<db::Translation, Box<dyn std::error::Error>> {
  let context = word_item.description.clone().unwrap();
  let translation = translate(&word_item.word, &context).await?;
  let description = translate(&context, "").await?;

  Ok(db::Translation {
    id: 0,
    word_id: word_item.id,
    translation: translation.to_lowercase(),
    description: Some(description),
    audio: None,
  })
}

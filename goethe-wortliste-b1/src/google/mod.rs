use crate::{db, utils};

use reqwest::{header, StatusCode};
use serde_json::{json, Value};
use std::env;

const GOOGLE_API_KEY_APP: &str = concat!("Bearer ", env!("GOOGLE_API_KEY_APP"));
const GOOGLE_API_KEY: &str = concat!("Bearer ", env!("GOOGLE_API_KEY"));
const GOOGLE_PROJECT: &str = env!("GOOGLE_PROJECT");

fn get_google_client(api_key: &str) -> reqwest::Client {
  let mut headers = header::HeaderMap::new();
  headers.insert(
    "Authorization",
    header::HeaderValue::from_str(api_key).unwrap(),
  );
  headers.insert(
    "x-goog-user-project",
    header::HeaderValue::from_static(GOOGLE_PROJECT),
  );

  reqwest::Client::builder()
    .default_headers(headers)
    .build()
    .unwrap()
}

pub async fn translate(word_item: &db::WordItem) -> Result<db::Translation, reqwest::Error> {
  let client = get_google_client(GOOGLE_API_KEY_APP);
  let map = json!({
      "contents": [word_item.word],
      "targetLanguageCode": "RU",
      "sourceLanguageCode": "DE",
  });
  let g_answer = client
    .post(format!(
      "https://translate.googleapis.com/v3beta1/projects/{}:translateText",
      GOOGLE_PROJECT
    ))
    .json(&map)
    .send()
    .await?;

  if g_answer.status() == StatusCode::OK {
    let data = g_answer.json::<serde_json::Value>().await?;
    let translated_text = data["translations"][0]["translatedText"].as_str().unwrap();

    let translation = db::Translation {
      id: 0,
      word_id: word_item.id,
      translation: String::from(translated_text),
      description: None,
      audio: None,
    };

    return Ok(translation);
  }

  panic!("Error: {:?}", g_answer)
}

pub async fn texttospeech(text: &str) -> Result<String, Box<dyn std::error::Error>> {
  let a_req_json = json!({
    "input": {
        "text": utils::extract_word_to_speach(text),
    },
    "voice": {
        "languageCode": "de-DE",
        "name": "de-DE-Wavenet-B"
    },
    "audioConfig": {
        "audioEncoding": "OGG_OPUS"
    }
  });

  let client = get_google_client(GOOGLE_API_KEY);
  let a_answer = client
    .post("https://texttospeech.googleapis.com/v1beta1/text:synthesize")
    .json(&a_req_json)
    .send()
    .await?
    .json::<Value>()
    .await?;

  let audio_content = a_answer["audioContent"].as_str().unwrap().to_string();

  Ok(audio_content)
}

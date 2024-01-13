use crate::utils;

use reqwest::header;
use serde_json::{json, Value};
use std::env;

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

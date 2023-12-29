use base64::decode;
use regex::Regex;
use reqwest::header;
use serde_json::{json, Value};
use std::{env, fs::File, io::Write};
use tokio::task::JoinSet;

use crate::db;

const GOOGLE_API_KEY: &str = env!("GOOGLE_API_KEY");
const GOOGLE_PROJECT: &str = env!("GOOGLE_PROJECT");

pub async fn translate() -> Result<(), Box<dyn std::error::Error>> {
  let connection = db::open();
  let words_query = "
    SELECT id, word FROM words
    WHERE id NOT IN (SELECT word_id FROM translations)
    LIMIT 10
  ";
  let mut set = JoinSet::new();

  for row in connection
    .prepare(words_query)?
    .into_iter()
    .map(|row| row.unwrap())
  {
    let mut word_item = db::WordItem {
      id: row.read::<i64, _>("id"),
      word: String::from(row.read::<&str, _>("word")),
    };
    let regex_pattern = Regex::new(r"^(\(.+\)) (?P<word>\w+)").unwrap();
    let Some(caps) = regex_pattern.captures(&word_item.word) else {
      continue;
    };
    word_item.word = String::from(caps.name("word").unwrap().as_str());
    set.spawn(async move { translate_google(&word_item).await.unwrap() });
  }

  while let Some(result) = set.join_next().await {
    match result {
      Ok(translation) => {
        let insert_translation_query =
          "INSERT INTO translations (word_id, translation) VALUES (:word_id, :translation)";
        let mut statement = connection.prepare(insert_translation_query).unwrap();

        statement
          .bind(
            &[
              (":word_id", translation.word_id.to_string().as_str()),
              (":translation", translation.translation.as_str()),
            ][..],
          )
          .unwrap();

        statement.next().unwrap();
      }
      Err(e) => println!("e: {}", e),
    }
  }

  Ok(())
}

async fn translate_google(word_item: &db::WordItem) -> Result<db::Translation, reqwest::Error> {
  let client = get_google_client();
  let map = json!({
      "contents": [word_item.word],
      "targetLanguageCode": "RU",
  });
  let g_answer = client
    .post(format!(
      "https://translate.googleapis.com/v3beta1/projects/{}:translateText",
      GOOGLE_PROJECT
    ))
    .json(&map)
    .send()
    .await?
    .json::<serde_json::Value>()
    .await?;

  let translated_text = g_answer["translations"][0]["translatedText"]
    .as_str()
    .unwrap();

  let translation = db::Translation {
    word_id: word_item.id,
    translation: String::from(translated_text),
  };

  Ok(translation)
}

fn get_google_client() -> reqwest::Client {
  let mut headers = header::HeaderMap::new();
  headers.insert(
    "Authorization",
    header::HeaderValue::from_static(GOOGLE_API_KEY),
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

pub async fn synthesize() -> Result<(), Box<dyn std::error::Error>> {
  let a_req_json = json!({
    "input": {
        "text": "Hallo Welt!"
    },
    "voice": {
        "languageCode": "de-DE",
        "name": "de-DE-Wavenet-B"
    },
    "audioConfig": {
        "audioEncoding": "MP3"
    }
  });

  let client = get_google_client();
  let a_answer = client
    .post("https://texttospeech.googleapis.com/v1beta1/text:synthesize")
    .json(&a_req_json)
    .send()
    .await?
    .json::<Value>()
    .await?;

  let b = a_answer["audioContent"].as_str().unwrap();
  let decoded_data = decode(b).unwrap();

  let file_path = "übrig.mp3";

  // Open the file in write mode
  let mut file = File::create(file_path)?;

  // Write the decoded data to the file
  file.write_all(&decoded_data)?;

  println!("Data saved to file: {}", file_path);

  Ok(())
}

use sql_query_builder as sql;
use std::env;
use tokio::task::JoinSet;

mod anki;
mod db;
mod deepl;
mod google;
mod openai;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let command = &args[1];

  if command == "init" {
    db::init_db();
  }

  if command == "clear" {
    db::clear_db()?;
  }

  if command == "seed" {
    db::seed_db()?;
  }

  if command == "generate_deck" {
    let data = db::read_words_and_translations()?;
    anki::generate_deck(&data)?;
  }

  if command == "translate" {
    translate_words().await?;
    synthesize().await?;
  }

  if command == "generate_image" {
    openai::image("").await?;
  }
  Ok(())
}

async fn translate_words() -> Result<(), Box<dyn std::error::Error>> {
  let mut set = JoinSet::new();
  let query = sql::Select::new()
    .limit("200")
    .select("id, word, description, lang")
    .where_clause("id NOT IN (SELECT word_id FROM translations)")
    .from("words");
  let words = db::read_words(&query.to_string());

  for mut word_item in words? {
    word_item.word = utils::extract_word_from_text(&word_item.word);
    set.spawn(async move { deepl::deep_translate(&word_item).await.unwrap() });
  }

  while let Some(result) = set.join_next().await {
    match result {
      Ok(translation) => db::insert_translation(&translation)?,
      Err(e) => println!("e: {}", e),
    }
  }

  Ok(())
}

async fn synthesize() -> Result<(), Box<dyn std::error::Error>> {
  let mut set = JoinSet::new();
  let translations_query = sql::Select::new()
    .select("*")
    .where_clause("audio IS NULL")
    .from("translations");
  let translations = db::read_translations(&translations_query.to_string());

  for mut translation in translations? {
    let word = db::read_word_item_by_id(translation.word_id);

    match word {
      Ok(word) => {
        set.spawn(async move {
          let speech = google::texttospeech(&word.word).await.unwrap();
          translation.audio = Some(speech);
          translation
        });
      }
      _ => (),
    }
  }
  while let Some(result) = set.join_next().await {
    match result {
      Ok(translation) => {
        db::update_translation(&translation)?;
      }
      Err(e) => println!("e: {}", e),
    }
  }
  Ok(())
}

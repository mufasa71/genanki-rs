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
    translate_description().await?;
  }

  if command == "generate_image" {
    openai::image("").await?;
  }
  Ok(())
}

async fn translate_words() -> Result<(), Box<dyn std::error::Error>> {
  let mut set = JoinSet::new();
  let words = db::read_words();

  for mut word_item in words? {
    word_item.word = utils::extract_word_from_text(&word_item.word);
    set.spawn(async move { google::translate(&word_item).await.unwrap() });
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
  let translations_query = sql::Select::new()
    .select("*")
    .where_clause("audio IS NULL")
    .from("translations");
  let translations = db::read_translations(&translations_query.to_string());

  for mut translation in translations? {
    let word = db::read_word_item_by_id(translation.word_id);

    match word {
      Ok(word) => {
        let speech = google::texttospeech(&word.word).await?;
        translation.audio = Some(speech);
        db::update_translation(&translation)?;
      }
      _ => (),
    }
  }
  Ok(())
}

async fn translate_description() -> Result<(), Box<dyn std::error::Error>> {
  let query = sql::Select::new()
    .select("*")
    .where_clause("description IS NULL")
    .from("translations");

  let translations = db::read_translations(&query.to_string());

  for mut translation in translations? {
    let word = db::read_word_item_by_id(translation.word_id);

    match word {
      Ok(word) => match word.description {
        Some(description) => {
          let translation_text = deepl::deep_translate(&description).await?;
          translation.description = Some(translation_text);
          db::update_translation(&translation)?;
        }
        _ => (),
      },
      e => println!("e: {:?}", e),
    }
  }

  Ok(())
}

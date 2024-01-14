use clap::{Arg, ArgAction, Command, Parser};
use sql_query_builder as sql;
use tokio::task::JoinSet;

mod anki;
mod db;
mod deepl;
mod google;
mod utils;

#[derive(Parser)]
struct Cli {
  command: String,
  model: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let matches = Command::new("b1")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
      Command::new("translate")
        .short_flag('T')
        .long_flag("translate")
        .about("Translate words in DB")
        .arg(
          Arg::new("speech")
            .short('s')
            .long("speech")
            .help("add text-to-speech")
            .action(ArgAction::Set)
            .num_args(1..),
        ),
    )
    .subcommand(
      Command::new("deck")
        .short_flag('D')
        .long_flag("deck")
        .about("Generate deck")
        .arg(
          Arg::new("model")
            .short('m')
            .long("model")
            .help("Anki model")
            .value_parser(["basic", "basic-reversed"])
            .action(ArgAction::Set)
            .require_equals(true)
            .default_value("basic")
            .default_missing_value("basic")
            .num_args(0..=1),
        ),
    )
    .subcommand(
      Command::new("init")
        .short_flag('I')
        .long_flag("init")
        .about("Initialize DB"),
    )
    .subcommand(
      Command::new("clear")
        .short_flag('C')
        .long_flag("clear")
        .about("Clear DB"),
    )
    .subcommand(
      Command::new("seed")
        .short_flag('S')
        .long_flag("seed")
        .about("Seed DB"),
    )
    .get_matches();

  match matches.subcommand() {
    Some(("translate", translate_matches)) => {
      translate_words().await?;
      if translate_matches.get_flag("speech") {
        synthesize().await?;
      }
    }
    Some(("deck", deck_matches)) => {
      let data = db::read_words_and_translations()?;
      let model = deck_matches
        .get_one::<String>("model")
        .map(|s| s.as_str())
        .expect("Please provide a model");

      if model == "basic" {
        anki::generate_deck(&data, &anki::basic_model())?;
      } else if model == "basic-reversed" {
        anki::generate_deck(&data, &anki::basic_model_reversed())?;
      }
    }
    Some(("init", _)) => db::init_db(),
    Some(("clear", _)) => db::clear_db()?,
    Some(("seed", _)) => db::seed_db()?,
    _ => unreachable!(),
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

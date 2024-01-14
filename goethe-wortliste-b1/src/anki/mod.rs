use crate::db;
use base64::decode;
use genanki_rs::{Deck, Error, Field, Model, Note, Package, Template};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::read_dir, fs::File, io::Write};

pub fn basic_model() -> Model {
  Model::new(
    1607392319,
    "German model (and reversed card)",
    vec![
      Field::new("AudioWord"),
      Field::new("WordTranslation"),
      Field::new("Word"),
      Field::new("Sentence"),
      Field::new("SentenceTranslation"),
      Field::new("AudioWordTranslation"),
      Field::new("AudioSentence"),
      Field::new("AudioSentenceTranslation"),
      Field::new("Level"),
      Field::new("CountryISO"),
      Field::new("Picture"),
      Field::new("Word-Symbol"),
      Field::new("Tags"),
      Field::new("Note/Mnemonic"),
      Field::new("CreateReversed"),
    ],
    vec![Template::new("Card 1")
      .qfmt(include_str!("template-q.html"))
      .afmt(include_str!("template-a.html"))],
  )
  .css(include_str!("minimal.css"))
}

pub fn basic_model_reversed() -> Model {
  let model = basic_model();
  model.with_template(
    Template::new("Card 2")
      .qfmt(include_str!("template-q-r.html"))
      .afmt(include_str!("template-a-r.html")),
  )
}

pub fn generate_deck(
  data: &Vec<(db::WordItem, db::Translation)>,
  model: &Model,
) -> Result<(), Error> {
  let mut my_deck = Deck::new(
    2059400110,
    "Goethe-Zertifikat B1",
    "Deutschprüfung für Jugendliche und Erwachsene",
  );
  let bar = ProgressBar::new(data.len() as u64);
  bar.set_style(
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
      .unwrap()
      .progress_chars("##-"),
  );
  for (word, translation) in data {
    bar.inc(1);
    match &translation.audio {
      Some(audio) => {
        let file_path = format!("audio/{}.ogg", word.id);
        if !std::path::Path::new(&file_path).exists() {
          let decoded_data = decode(audio).unwrap();
          let mut file = File::create(&file_path)?;
          file.write_all(&decoded_data)?;
        }
      }
      _ => (),
    }
    let description = translation.description.clone().unwrap();
    let word_translation = word.description.clone().unwrap();
    let my_note = Note::new(
      model.clone(),
      vec![
        &format!("[sound:{}.ogg]", word.id),
        &translation.translation,
        &word.word,
        &word_translation,
        &description,
        "",
        "",
        "",
        "",
        "DE",
        "",
        "",
        "",
        "",
        "true",
      ],
    )?;
    my_deck.add_note(my_note);
  }
  bar.finish();

  let mut files = vec![
    "media/_flag_de.svg".to_string(),
    "media/flag_de.svg".to_string(),
    "media/germany-flag-1783774.svg".to_string(),
  ];
  for entry in read_dir("audio").unwrap() {
    let path = entry.unwrap().path().to_str().unwrap().to_string();
    files.push(path);
  }

  let files = files.iter().map(String::as_str).collect::<Vec<_>>();

  let mut my_package = Package::new(vec![my_deck], files)?;

  my_package.write_to_file("goethe-zertifikat-b1.apkg")
}

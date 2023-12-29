use scraper::{Html, Selector};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WordItem {
  pub id: i64,
  pub word: String,
}

#[derive(Debug)]
pub struct Translation {
  pub word_id: i64,
  pub translation: String,
}

pub fn open() -> sqlite::Connection {
  let connection = sqlite::open("wortlist.sqlite").unwrap();

  connection
}

pub fn init_db() {
  let connection = open();

  let query = "
    CREATE TABLE IF NOT EXISTS config (
        hash TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS words (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        lang TEXT NOT NULL,
        word TEXT NOT NULL UNIQUE,
        description TEXT
    );
    CREATE TABLE IF NOT EXISTS translations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        word_id INTEGER NOT NULL UNIQUE,
        translation TEXT NOT NULL,
        FOREIGN KEY (word_id) REFERENCES words(id)
    );
  ";

  connection.execute(query).unwrap();
}

pub fn seed_db() -> Result<(), Box<dyn std::error::Error>> {
  let bytes = std::fs::read("./all_b1.html")?;
  let hash = sha256::digest(&bytes);

  let connection = open();
  let drop_all_query = "
    DELETE FROM config;
    DELETE FROM words;
    DELETE FROM translations;
    ";
  connection.execute(drop_all_query)?;
  let check_hash_query = "
    SELECT hash
    FROM config
    WHERE hash = :hash
  ";
  let mut statement = connection.prepare(check_hash_query)?;
  statement.bind((":hash", hash.as_str()))?;

  if statement.iter().count() == 0 {
    let insert_hash_query = "
        INSERT INTO config (hash)
        VALUES (:hash)
    ";

    let mut statement = connection.prepare(insert_hash_query)?;
    statement.bind((":hash", hash.as_str()))?;
    statement.next()?;

    let wortlist = read_wortlist_from_file();
    let insert_words_query = "
        INSERT INTO words (lang, word, description)
        VALUES (:lang, :word, :description)
    ";

    for (word, description) in wortlist {
      let mut statement = connection.prepare(insert_words_query)?;
      statement.bind(
        &[
          (":lang", "DE"),
          (":word", word.as_str()),
          (":description", description.join("\n").as_str()),
        ][..],
      )?;
      statement.next()?;
    }
  }

  Ok(())
}

fn read_wortlist_from_file() -> HashMap<String, Vec<String>> {
  let html_file = std::fs::read_to_string("all_b1.html").unwrap();
  let document = Html::parse_document(&html_file);
  let tr = Selector::parse("tr").unwrap();
  let td = Selector::parse("td").unwrap();
  let mut dict = HashMap::new();

  for row in document.select(&tr) {
    let mut key = "";
    let mut value: Vec<String> = vec![];

    for (i, cell) in row.select(&td).enumerate() {
      if i == 0 {
        key = cell.text().next().unwrap();
        continue;
      }

      for t in cell.text() {
        value.push(String::from(t));
      }
    }
    dict.insert(String::from(key), value);
  }

  dict
}

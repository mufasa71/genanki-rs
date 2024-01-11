use rusqlite::{params, Connection, Result};
use scraper::{Html, Selector};
use sql_query_builder as sql;
use std::collections::HashMap;

#[derive(Debug)]
pub struct WordItem {
  pub id: i64,
  pub word: String,
  pub description: Option<String>,
  pub lang: String,
}

#[derive(Debug)]
pub struct Translation {
  pub id: i64,
  pub word_id: i64,
  pub translation: String,
  pub description: Option<String>,
  pub audio: Option<String>,
}

pub fn open() -> Connection {
  let connection = Connection::open("wortlist.sqlite").unwrap();

  connection
}

pub fn init_db() {
  let connection = open();

  let query = "
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
        description TEXT,
        audio TEXT,
        FOREIGN KEY (word_id) REFERENCES words(id)
    );
  ";

  connection.execute(query, []).unwrap();
}

pub fn clear_db() -> Result<()> {
  let conn = open();
  let delete_words_query = sql::Delete::new().delete_from("words");
  let delete_translations_query = sql::Delete::new().delete_from("translations");
  conn.execute(&delete_translations_query.to_string(), [])?;
  conn.execute(&delete_words_query.to_string(), [])?;

  Ok(())
}

pub fn seed_db() -> Result<(), Box<dyn std::error::Error>> {
  let connection = open();
  let wortlist = read_wortlist_from_file();
  let insert_query = sql::Insert::new()
    .insert_into("words (lang, word, description)")
    .values("(?1, ?2, ?3)");

  for (word, description) in wortlist {
    connection.execute(
      &insert_query.to_string(),
      params!["DE", word, description.join("")],
    )?;
  }

  Ok(())
}

pub fn read_words() -> Result<Vec<WordItem>, rusqlite::Error> {
  let query = sql::Select::new()
    .limit("200")
    .select("id, word, description, lang")
    .where_clause("id NOT IN (SELECT word_id FROM translations)")
    .from("words");
  let conn = open();
  let mut smt = conn.prepare(&query.to_string())?;
  let rows = smt.query_map([], |row| {
    Ok(WordItem {
      id: row.get(0)?,
      word: row.get(1)?,
      description: row.get(2)?,
      lang: row.get(3)?,
    })
  })?;
  let mut words = vec![];

  for row in rows {
    words.push(row?);
  }

  Ok(words)
}

pub fn read_word_item_by_id(id: i64) -> Result<WordItem> {
  let word_query = sql::Select::new()
    .select("id, word, description, lang")
    .from("words")
    .where_clause("id = ?1");
  let conn = open();
  let mut smt = conn.prepare(&word_query.to_string())?;
  smt.query_row(params![id], |row| {
    Ok(WordItem {
      id: row.get(0)?,
      word: row.get(1)?,
      description: row.get(2)?,
      lang: row.get(3)?,
    })
  })
}

pub fn read_translations(query: &str) -> Result<Vec<Translation>> {
  let conn = open();
  let mut smt = conn.prepare(query)?;
  let rows = smt.query_map([], |row| {
    Ok(Translation {
      id: row.get(0)?,
      word_id: row.get(1)?,
      translation: row.get(2)?,
      description: row.get(3)?,
      audio: row.get(4)?,
    })
  })?;
  let mut translations = vec![];

  for row in rows {
    translations.push(row?);
  }

  Ok(translations)
}

pub fn read_words_and_translations() -> Result<Vec<(WordItem, Translation)>> {
  let query = sql::Select::new()
    .select(
      "w.id, w.word, w.description, w.lang, t.id, t.word_id, t.translation, t.description, t.audio",
    )
    .from("translations t")
    .inner_join("words w ON t.word_id = w.id");
  let conn = open();
  let mut smt = conn.prepare(&query.to_string())?;
  let rows = smt.query_map([], |row| {
    Ok((
      WordItem {
        id: row.get(0)?,
        word: row.get(1)?,
        description: row.get(2)?,
        lang: row.get(3)?,
      },
      Translation {
        id: row.get(4)?,
        word_id: row.get(5)?,
        translation: row.get(6)?,
        description: row.get(7)?,
        audio: row.get(8)?,
      },
    ))
  })?;
  let mut words = vec![];

  for row in rows {
    words.push(row?);
  }
  Ok(words)
}

pub fn update_translation(translation: &Translation) -> Result<()> {
  let update_query = sql::Update::new()
    .update("translations")
    .set("audio = ?1")
    .set("description = ?2")
    .where_clause("id = ?3");

  open().execute(
    &update_query.to_string(),
    params![translation.audio, translation.description, translation.id],
  )?;

  Ok(())
}

pub fn insert_translation(translation: &Translation) -> Result<()> {
  let insert_query = sql::Insert::new()
    .insert_into("translations (word_id, translation)")
    .values("(?1, ?2)");

  open().execute(
    &insert_query.to_string(),
    params![translation.word_id, translation.translation],
  )?;

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

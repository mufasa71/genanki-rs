use anyhow::Result;
use scraper::{Html, Selector};
use sql_query_builder as sql;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{collections::HashMap, env};

#[derive(Debug)]
pub struct WordItem {
    pub id: i64,
    pub word: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct Translation {
    pub id: i64,
    pub word_id: i64,
    pub translation: String,
    pub description: Option<String>,
    pub audio: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
pub async fn open() -> Result<Pool<Sqlite>> {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    Ok(pool)
}

pub async fn init_db() -> Result<()> {
    let pool = open()?;
    let mut conn = pool.acquire().await?;

    sqlx::query(
        r#"
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
  "#,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn clear_db() -> Result<()> {
    let pool = open()?;
    let mut conn = pool.acquire().await?;
    let delete_translations_query = sql::Delete::new().delete_from("translations");

    sqlx::query(&delete_translations_query.as_string())
        .execute(&mut *conn)
        .await?;

    Ok(())
}

pub async fn seed_db() -> Result<()> {
    let pool = open()?;
    let mut conn = pool.acquire().await?;
    let wortlist = read_wortlist_from_file();

    for (word, description) in wortlist {
        let description = description.join("");
        sqlx::query!(
            r#"
                INSERT INTO words (lang, word, description) VALUES (?, ?, ?)
            "#,
            "DE",
            word,
            description
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

pub async fn read_words() -> Result<Vec<WordItem>> {
    let pool = open()?;
    let recs = sqlx::query!(
        r#"
        SELECT id, word, description, lang
        FROM words
        WHERE id NOT IN (SELECT word_id FROM translations)
        LIMIT 200
    "#
    )
    .map(|row| WordItem {
        id: row.id,
        description: row.description,
        word: row.word,
    })
    .fetch_all(&pool)
    .await?;

    Ok(recs)
}

pub async fn read_word_item_by_id(id: i64) -> Result<WordItem> {
    let pool = open()?;
    let rec = sqlx::query!(
        r#"
        SELECT id, word, description, lang
        FROM words
        WHERE id = ?
    "#,
        id
    )
    .map(|row| WordItem {
        id: row.id,
        word: row.word,
        description: row.description,
    })
    .fetch_one(&pool)
    .await?;

    Ok(rec)
}

pub async fn read_translations() -> Result<Vec<Translation>> {
    let pool = open()?;
    let recs = sqlx::query!(
        r#"
    SELECT *
    FROM translations
    WHERE audio IS NULL
    "#
    )
    .map(|row| Translation {
        id: row.id,
        word_id: row.word_id,
        translation: row.translation,
        description: row.description,
        audio: row.audio.clone(),
    })
    .fetch_all(&pool)
    .await?;

    Ok(recs)
}

pub async fn read_words_and_translations() -> Result<Vec<(WordItem, Translation)>> {
    let pool = open()?;
    let recs = sqlx::query!(r#"
        SELECT w.id, w.word, w.description as w_description, w.lang, t.id as t_id, t.word_id, t.translation, t.description, t.audio
        FROM translations t
        INNER JOIN words w ON t.word_id = w.id
    "#).map(|row| (
            WordItem {
                id: row.id,
                word: row.word,
                description: row.w_description,
            },
            Translation {
                id: row.t_id,
                word_id: row.id,
                translation: row.translation,
                description: row.description,
                audio: row.audio,
            },

        )).fetch_all(&pool).await?;

    Ok(recs)
}

pub async fn update_translation(translation: &Translation) -> Result<()> {
    let pool = open()?;
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE translations SET audio = ?, description = ?
        WHERE id = ?
    "#,
        translation.audio,
        translation.description,
        translation.id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn insert_translation(translation: &Translation) -> Result<()> {
    let pool = open()?;
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        INSERT INTO translations (word_id, translation, description)
        VALUES (?, ?, ?)
    "#,
        translation.word_id,
        translation.translation,
        translation.description
    )
    .execute(&mut *conn)
    .await?;

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

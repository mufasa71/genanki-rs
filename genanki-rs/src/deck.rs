use super::Package;
use crate::db_entries::{DeckDbEntry, ModelDbEntry};
use crate::error::json_error;
use crate::model::Model;
use crate::note::Note;
use anyhow::Result;
use sqlx::SqliteConnection;
use std::collections::HashMap;
use std::path::Path;

/// A flashcard deck which can be written into an .apkg file.
#[derive(Clone)]
pub struct Deck {
    id: i64,
    name: String,
    description: String,
    notes: Vec<Note>,
    models: HashMap<i64, Model>,
}

impl Deck {
    /// Creates a new deck with an `id`, `name` and `description`.
    ///
    /// `id` should always be unique when creating multiple decks.
    pub fn new(id: i64, name: &str, description: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            notes: vec![],
            models: HashMap::new(),
        }
    }

    /// Adds a `note` (Flashcard) to the deck.
    ///
    /// Example:
    ///
    /// ```rust
    /// use genanki_rs::{Deck, Note, basic_model};
    /// use anyhow::Result;
    ///
    /// #[tokio::main] async fn main() -> Result<()> {
    /// let mut my_deck = Deck::new(1234, "Example deck", "This is an example deck");
    /// my_deck.add_note(Note::new(basic_model(), vec!["What is the capital of France?", "Paris"])?);
    ///
    /// Ok(())
    /// }
    /// ```
    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    fn add_model(&mut self, model: Model) {
        self.models.insert(model.id, model);
    }

    fn to_deck_db_entry(&self) -> DeckDbEntry {
        DeckDbEntry {
            collapsed: false,
            conf: 1,
            desc: self.description.clone(),
            deck_db_entry_dyn: 0,
            extend_new: 0,
            extend_rev: 50,
            id: self.id,
            lrn_today: vec![163, 2],
            deck_db_entry_mod: 1425278051,
            name: self.name.clone(),
            new_today: vec![163, 2],
            rev_today: vec![163, 0],
            time_today: vec![163, 23598],
            usn: -1,
        }
    }

    #[allow(dead_code)]
    fn to_json(&self) -> String {
        let db_entry: DeckDbEntry = self.to_deck_db_entry();
        serde_json::to_string(&db_entry).expect("Should always serialize")
    }

    pub(super) async fn write_to_db(
        &mut self,
        conn: &mut SqliteConnection,
        timestamp: f64,
    ) -> Result<()> {
        let rec = sqlx::query!(
            r#"
            SELECT decks FROM col
        "#
        )
        .fetch_one(&mut *conn)
        .await?;

        let mut decks: HashMap<i64, DeckDbEntry> =
            serde_json::from_str(&rec.decks).map_err(json_error)?;

        decks.insert(self.id, self.to_deck_db_entry());

        let decks_string = serde_json::to_string(&decks)?.clone();
        sqlx::query!(
            r#"
                UPDATE col SET decks = ?
        "#,
            decks_string
        )
        .execute(&mut *conn)
        .await?;

        let models_json_str = sqlx::query!(
            r#"
            SELECT models FROM col
        "#
        )
        .fetch_one(&mut *conn)
        .await?;

        let mut models: HashMap<i64, ModelDbEntry> =
            serde_json::from_str(&models_json_str.models).map_err(json_error)?;
        for note in self.notes.clone().iter() {
            self.add_model(note.model());
        }
        for (i, model) in &mut self.models {
            models.insert(*i, model.to_model_db_entry(timestamp, self.id)?);
        }
        let models_string = serde_json::to_string(&models)?.clone();
        let _ = sqlx::query!(
            r#"
                UPDATE col SET models = ?
        "#,
            models_string
        )
        .execute(&mut *conn)
        .await?;

        for note in &mut self.notes {
            note.write_to_db(&mut *conn, timestamp, self.id).await?;
        }
        Ok(())
    }

    /// Packages a deck and writes it to a new `.apkg` file. This file can then be imported in Anki.
    ///
    /// Returns `Err` if the file can not be created.
    ///
    /// Example:
    /// ```rust
    /// use genanki_rs::{Deck, Note, basic_model};
    /// use anyhow::Result;
    ///
    /// #[tokio::main] async fn main() -> Result<()> {
    /// let mut my_deck = Deck::new(1234, "Example deck", "This is an example deck");
    /// my_deck.add_note(Note::new(basic_model(), vec!["What is the capital of France?", "Paris"])?);
    ///
    /// my_deck.generate_anki("output.apkg").await?;
    ///
    /// Ok(())
    /// }
    /// ```
    ///
    /// This is equivalent to:
    /// ```rust
    /// use genanki_rs::{Deck, Note, basic_model, Package};
    /// use anyhow::Result;
    ///
    /// #[tokio::main] async fn main() -> Result<()> {
    /// let mut my_deck = Deck::new(1234, "Example deck", "This is an example deck");
    /// my_deck.add_note(Note::new(basic_model(), vec!["What is the capital of France?", "Paris"])?);
    ///
    /// Package::new(vec![my_deck], vec![])?.generate_anki("output.apkg", None).await?;
    ///
    /// Ok(())
    /// }
    /// ```
    pub async fn generate_anki(&self, file: &str) -> Result<()> {
        Package::new(vec![self.clone()], vec![])?
            .generate_anki(file, None)
            .await?;
        Ok(())
    }

    pub async fn write_to_file(
        &self,
        file: &str,
        conn: &mut SqliteConnection,
        db_file_path: &Path,
    ) -> Result<()> {
        Package::new(vec![self.clone()], vec![])?
            .write_to_file(file, conn, db_file_path)
            .await?;
        Ok(())
    }
}

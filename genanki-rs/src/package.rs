use sqlx::SqliteConnection;
use sqlx::migrate::MigrateDatabase;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::NamedTempFile;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use crate::deck::Deck;
use crate::error::{json_error, zip_error};
use anyhow::Result;
use std::str::FromStr;

/// `Package` to pack `Deck`s and `media_files` and write them to a `.apkg` file
///
/// Example:
/// ```rust
/// use genanki_rs::{Package, Deck, Note, Model, Field, Template};
/// use anyhow::Result;
///
///#[tokio::main] async fn main() -> Result<()> {
/// let model = Model::new(
///     1607392319,
///     "Simple Model",
///     vec![
///         Field::new("Question"),
///         Field::new("Answer"),
///         Field::new("MyMedia"),
///     ],
///     vec![Template::new("Card 1")
///         .qfmt("{{Question}}{{Question}}<br>{{MyMedia}}")
///         .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#)],
/// );
///
/// let mut my_deck = Deck::new(1234, "Example Deck", "Example Deck with media");
/// my_deck.add_note(Note::new(model.clone(), vec!["What is the capital of France?", "Paris", "[sound:sound.mp3]"])?);
/// my_deck.add_note(Note::new(model.clone(), vec!["What is the capital of France?", "Paris", r#"<img src="image.jpg">"#])?);
///
/// let mut package = Package::new(vec![my_deck], vec!["fixtures/sound.mp3", "fixtures/image.jpg"])?;
/// package.generate_anki("output.apkg", None).await?;
/// Ok(())
/// }
/// ```
pub struct Package {
    decks: Vec<Deck>,
    media_files: Vec<PathBuf>,
}

impl Package {
    /// Create a new package with `decks` and `media_files`
    ///
    /// Returns `Err` if `media_files` are invalid
    pub fn new(decks: Vec<Deck>, media_files: Vec<&str>) -> Result<Self> {
        let media_files = media_files
            .iter()
            .map(|&s| PathBuf::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { decks, media_files })
    }

    /// Writes the package to any writer that implements Write and Seek
    pub async fn write<W: Write + Seek>(
        &mut self,
        writer: W,
        conn: &mut SqliteConnection,
        db_file_path: &Path,
    ) -> Result<()> {
        self.write_maybe_timestamp(None, &mut *conn).await?;
        self.write_to_zip(writer, db_file_path)?;

        Ok(())
    }

    /// Writes the package to any writer that implements Write and Seek using a timestamp
    pub async fn write_timestamp<W: Write + Seek>(
        &mut self,
        writer: W,
        timestamp: f64,
        conn: &mut SqliteConnection,
        db_file_path: &Path,
    ) -> Result<()> {
        self.write_maybe_timestamp(Some(timestamp), &mut *conn)
            .await?;
        self.write_to_zip(writer, db_file_path)?;

        Ok(())
    }

    /// Writes the package to a file
    ///
    /// Returns `Err` if the `file` cannot be created
    pub async fn write_to_file(
        &mut self,
        file: &str,
        conn: &mut SqliteConnection,
        db_file_path: &Path,
    ) -> Result<()> {
        let file = File::create(file)?;
        self.write_maybe_timestamp(None, conn).await?;
        self.write_to_zip(file, db_file_path)?;

        Ok(())
    }

    /// Writes the package to a file using a timestamp
    ///
    /// Returns `Err` if the `file` cannot be created
    pub async fn write_to_file_timestamp(
        &mut self,
        file: &str,
        timestamp: Option<f64>,
        conn: &mut SqliteConnection,
        db_file_path: &Path,
    ) -> Result<()> {
        let file = File::create(file)?;
        self.write_maybe_timestamp(timestamp, conn).await?;
        self.write_to_zip(file, db_file_path)?;

        Ok(())
    }

    pub async fn write_maybe_timestamp(
        &mut self,
        timestamp: Option<f64>,
        conn: &mut SqliteConnection,
    ) -> Result<()> {
        let timestamp = if let Some(timestamp) = timestamp {
            timestamp
        } else {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64()
        };

        for deck in &mut self.decks {
            deck.write_to_db(&mut *conn, timestamp).await?;
        }

        Ok(())
    }

    pub fn write_to_zip<W: Write + Seek>(&mut self, writer: W, db_file_path: &Path) -> Result<()> {
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        let mut outzip = ZipWriter::new(writer);
        outzip
            .start_file("collection.anki2", options)
            .map_err(zip_error)?;
        outzip.write_all(&read_file_bytes(db_file_path)?)?;

        let media_file_idx_to_path = self
            .media_files
            .iter()
            .enumerate()
            .collect::<HashMap<usize, &PathBuf>>();
        let media_map = media_file_idx_to_path
            .clone()
            .into_iter()
            .map(|(id, path)| {
                (
                    id.to_string(),
                    path.file_name()
                        .expect("Should always have a filename")
                        .to_str()
                        .expect("should always have string"),
                )
            })
            .collect::<HashMap<String, &str>>();
        let media_json = serde_json::to_string(&media_map).map_err(json_error)?;
        outzip.start_file("media", options).map_err(zip_error)?;
        outzip.write_all(media_json.as_bytes())?;

        for (idx, &path) in &media_file_idx_to_path {
            outzip
                .start_file(idx.to_string(), options)
                .map_err(zip_error)?;
            outzip.write_all(&read_file_bytes(path)?)?;
        }
        outzip.finish().map_err(zip_error)?;

        Ok(())
    }

    pub async fn generate_anki(&mut self, file_name: &str, timestamp: Option<f64>) -> Result<()> {
        let db_tmp_file = NamedTempFile::new()?;
        let db_file_path = db_tmp_file.path();
        let db_file_url = db_file_path.to_str().expect("DB file should be created");
        sqlx::Sqlite::create_database(db_file_url).await?;

        let pool = sqlx::SqlitePool::connect(db_file_url).await?;
        let mut conn = pool.acquire().await?;
        sqlx::migrate!().run(&mut *conn).await?;
        sqlx::query_file!("fixtures/anki.sql")
            .execute(&mut *conn)
            .await?;

        self.write_to_file_timestamp(file_name, timestamp, &mut conn, db_file_path)
            .await?;

        Ok(())
    }
}

fn read_file_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut handle = File::open(path)?;
    let mut data = Vec::new();
    handle.read_to_end(&mut data)?;
    Ok(data)
}

use crate::Error;
use crate::card::Card;
use crate::model::{Model, ModelType};
use crate::util::guid_for;
use anyhow::{Result, anyhow};
use fancy_regex::Regex;
use sqlx::SqliteConnection;
use std::collections::HashSet;
use std::str::FromStr;

/// Note (Flashcard) to be added to a `Deck`
#[derive(Clone)]
pub struct Note {
    model: Model,
    fields: Vec<String>,
    sort_field: bool,
    tags: Vec<String>,
    guid: String,
    cards: Vec<Card>,
}

impl Note {
    /// Creates a new Note with a new `model` and `fields`
    ///
    /// Returns `Err` if the fields are not matching the model or if the fields are invalid
    ///
    /// Example:
    /// ```
    /// use genanki_rs::{Note, basic_model};
    ///
    /// let note = Note::new(basic_model(), vec!["What is the capital of France?", "Paris"]);
    /// ```
    pub fn new(model: Model, fields: Vec<&str>) -> Result<Self> {
        let fields: Vec<String> = fields.iter().map(|&s| s.to_string()).collect();

        let cards = match model.get_model_type() {
            ModelType::FrontBack => front_back_cards(&model, &fields)?,
            ModelType::Cloze => cloze_cards(&model, &fields),
        };
        let guid = guid_for(&fields);
        Ok(Self {
            model,
            fields,
            sort_field: false,
            tags: vec![],
            guid,
            cards,
        })
    }

    /// Creates a new Note with a new `model`, `fields` and custom parameters:
    /// * `sort_field` - whether to sort field, default is `false`
    /// * `tags` - List of tags
    /// * `guid` - Custom unique note id, default is hash of all fields
    ///
    /// Returns `Err` if tags or fields are invalid
    pub fn new_with_options(
        model: Model,
        fields: Vec<&str>,
        sort_field: Option<bool>,
        tags: Option<Vec<&str>>,
        guid: Option<&str>,
    ) -> Result<Self> {
        let tags: Vec<String> = tags
            .unwrap_or_default()
            .iter()
            .map(|s| s.to_string())
            .collect();
        validate_tags(&tags)?;
        let fields: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
        let cards = match model.get_model_type() {
            ModelType::FrontBack => front_back_cards(&model, &fields)?,
            ModelType::Cloze => cloze_cards(&model, &fields),
        };
        let guid = guid.unwrap_or(&guid_for(&fields)).to_string();
        Ok(Self {
            model,
            fields,
            sort_field: sort_field.unwrap_or(false),
            tags,
            guid,
            cards,
        })
    }

    /// Returns a new Note with the sort field replace with the new one
    pub fn sort_field(self, sort_field: bool) -> Self {
        Self { sort_field, ..self }
    }

    /// Sets or replaces tags with the provided ones
    pub fn tags(self, tags: impl IntoIterator<Item = impl ToString>) -> Self {
        Self {
            tags: tags.into_iter().map(|tag| tag.to_string()).collect(),
            ..self
        }
    }

    /// Adds an additional tag
    pub fn with_tag(mut self, tag: impl ToString) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Sets the GUID for this note
    ///
    /// The GUID is auto-generated if this option is not provided.
    pub fn guid(self, guid: impl ToString) -> Self {
        Self {
            guid: guid.to_string(),
            ..self
        }
    }

    pub(super) fn model(&self) -> Model {
        self.model.clone()
    }

    #[allow(dead_code)]
    pub(super) fn cards(&self) -> Vec<Card> {
        self.cards.clone()
    }

    fn get_guid(&self) -> String {
        self.guid.clone()
    }

    fn check_number_model_fields_matches_num_fields(&self) -> Result<()> {
        if self.model.fields().len() != self.fields.len() {
            Err(anyhow!(Error::ModelFieldCountMismatch {
                model_len: self.model.fields().len(),
                card_len: self.fields.len()
            }))
        } else {
            Ok(())
        }
    }

    fn check_invalid_html_tags_in_fields(&self) -> Result<()> {
        for field in &self.fields {
            let invalid_tags = find_invalid_html_tags_in_field(field);
            if !invalid_tags.is_empty() {
                println!(
                    "Warning: The field {} contains the invalid html tags {:?}",
                    field, invalid_tags
                );
            }
        }
        Ok(())
    }

    fn format_fields(&self) -> String {
        self.fields.clone().join("\x1f")
    }

    fn format_tags(&self) -> String {
        format!(" {} ", self.tags.join(" "))
    }
    pub(super) async fn write_to_db(
        &self,
        conn: &mut SqliteConnection,
        timestamp: f64,
        deck_id: i64,
    ) -> Result<()> {
        self.check_number_model_fields_matches_num_fields()?;
        self.check_invalid_html_tags_in_fields()?;
        // let mut conn = pool.acquire().await?;
        let guid = self.get_guid();
        let timestamp_i64 = timestamp as i64;
        let tags = self.format_tags();
        let flds = self.format_fields();

        let rec = sqlx::query!(
            r#"
                INSERT INTO notes (guid, mid, mod, usn, tags, flds, sfld, csum, flags, data)
                VALUES(?,?,?,?,?,?,?,?,?,?)
        "#,
            guid,
            self.model.id, // mid
            timestamp_i64,
            -1, // usn
            tags,
            flds,
            self.sort_field, // sfld
            0,               // csum, can be ignored
            0,               // flags
            "",              // data
        )
        .execute(&mut *conn)
        .await?;
        let note_id = rec.last_insert_rowid() as usize;

        for card in &self.cards {
            card.write_to_db(conn, timestamp, deck_id, note_id).await?;
        }
        Ok(())
    }
}

fn cloze_cards(model: &Model, self_fields: &[String]) -> Vec<Card> {
    let mut card_ords: HashSet<i64> = HashSet::new();
    let mut cloze_replacements: HashSet<String> = HashSet::new();
    cloze_replacements.extend(re_findall(
        r"{{[^}]*?cloze:(?:[^}]?:)*(.+?)}}",
        &model.templates()[0].qfmt,
    ));
    cloze_replacements.extend(re_findall("<%cloze:(.+?)%>", &model.templates()[0].qfmt));
    for field_name in cloze_replacements {
        let fields = model.fields();
        let mut field_index_iter = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.name == field_name)
            .map(|(i, _)| i);
        let field_value = if let Some(field_index) = field_index_iter.next() {
            self_fields[field_index].clone()
        } else {
            "".to_string()
        };
        let updates_str = re_findall(r"(?s){{c(\d+)::.+?}}", &field_value);
        let updates = updates_str
            .iter()
            .map(|m| i64::from_str(m).expect("parsed from regex") - 1)
            .filter(|&m| m >= 0);
        card_ords.extend(updates);
    }
    if card_ords.is_empty() {
        card_ords.insert(0);
    }
    card_ords
        .iter()
        .map(|&card_ord| Card::new(card_ord, false))
        .collect()
}

fn front_back_cards(model: &Model, self_fields: &[String]) -> Result<Vec<Card>> {
    let mut rv = vec![];
    for (card_ord, any_or_all, required_field_ords) in model.req()?.drain(..) {
        let mut iter = required_field_ords.iter().map(|&ord| &self_fields[ord]);
        let condition = match any_or_all.as_str() {
            "any" => iter.any(|field| !field.is_empty()),
            "all" => iter.all(|field| !field.is_empty()),
            _ => panic!("only any or all"),
        };
        if condition {
            rv.push(Card::new(card_ord as i64, false));
        }
    }
    Ok(rv)
}

fn re_findall(regex_str: &'static str, to_match: &str) -> Vec<String> {
    let regex = Regex::new(regex_str).expect("static regex");
    regex
        .captures_iter(to_match)
        .filter_map(|m| m.ok())
        .flat_map(|cap| {
            cap.iter()
                .skip(1)
                .flatten()
                .map(|m| m.as_str().to_string())
                .collect::<Vec<String>>()
        })
        .collect()
}

fn validate_tags(tags: &[String]) -> Result<()> {
    if tags.iter().any(|tag| tag.contains(' ')) {
        Err(anyhow!(Error::TagContainsWhitespace))
    } else {
        Ok(())
    }
}

fn find_invalid_html_tags_in_field(field: &str) -> Vec<String> {
    let regex = Regex::new(r"<(?!/?[a-z0-9]+(?: .*|/?)>)(?:.|\n)*?>").unwrap();
    regex
        .find_iter(field)
        .map(|m| m.unwrap().as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use super::*;
    use crate::{Field, Model, Note, Template};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_to_db_setup() -> (f64, i64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        (timestamp, 0)
    }

    #[sqlx::test(fixtures("anki"))]
    async fn ok(pool: Pool<Sqlite>) {
        let my_model = Model::new(
            1376484377,
            "Simple Model",
            vec![Field::new("Question"), Field::new("Answer")],
            vec![
                Template::new("Card 1")
                    .qfmt("{{Question}}")
                    .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#),
            ],
        );
        let my_note = Note::new(my_model, vec!["Capital of Argentina", "Buenos Aires"]).unwrap();
        let (timestamp, deck_id) = write_to_db_setup();

        let mut conn = pool.acquire().await.unwrap();
        my_note
            .write_to_db(&mut conn, timestamp, deck_id)
            .await
            .unwrap();
    }

    #[test]
    fn tags_new() {
        let _ = Note::new_with_options(
            Model::new(0, "test", vec![], vec![]),
            vec![],
            None,
            Some(vec!["foo", "bar", "baz"]),
            None,
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn tags_new_panic() {
        let _ = Note::new_with_options(
            Model::new(0, "test", vec![], vec![]),
            vec![],
            None,
            Some(vec!["fo o", "bar", "baz"]),
            None,
        )
        .unwrap();
    }

    #[sqlx::test(fixtures("anki"))]
    async fn num_fields_equals_model_ok(pool: Pool<Sqlite>) {
        let model = Model::new(
            1894808898,
            "Test Model",
            vec![
                Field::new("Question"),
                Field::new("Answer"),
                Field::new("Extra"),
            ],
            vec![
                Template::new("Card 1")
                    .qfmt("{{Question}}")
                    .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#),
            ],
        );

        let note = Note::new(
            model,
            vec![
                "Capital of Germany",
                "Berlin",
                "Berlin was divided by a wall until 1989",
            ],
        )
        .unwrap();
        let (timestamp, deck_id) = write_to_db_setup();
        let mut conn = pool.acquire().await.unwrap();
        note.write_to_db(&mut conn, timestamp, deck_id)
            .await
            .unwrap();
    }

    #[sqlx::test(fixtures("anki"))]
    #[should_panic]
    async fn num_fields_less_than_model_panic(pool: Pool<Sqlite>) {
        let model = Model::new(
            1894808898,
            "Test Model",
            vec![
                Field::new("Question"),
                Field::new("Answer"),
                Field::new("Extra"),
            ],
            vec![
                Template::new("Card 1")
                    .qfmt("{{Question}}")
                    .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#),
            ],
        );

        let note = Note::new(model, vec!["Capital of Germany", "Berlin"]).unwrap();
        let (timestamp, deck_id) = write_to_db_setup();
        let mut conn = pool.acquire().await.unwrap();
        note.write_to_db(&mut conn, timestamp, deck_id)
            .await
            .unwrap();
    }

    #[sqlx::test(fixtures("anki"))]
    #[should_panic]
    async fn num_fields_more_than_model_panic(pool: Pool<Sqlite>) {
        let model = Model::new(
            1894808898,
            "Test Model",
            vec![Field::new("Question"), Field::new("Answer")],
            vec![
                Template::new("Card 1")
                    .qfmt("{{Question}}")
                    .afmt(r#"{{FrontSide}}<hr id="answer">{{Answer}}"#),
            ],
        );

        let note = Note::new(
            model,
            vec![
                "Capital of Germany",
                "Berlin",
                "Berlin was divided by a wall until 1989",
            ],
        )
        .unwrap();
        let (timestamp, deck_id) = write_to_db_setup();
        let mut conn = pool.acquire().await.unwrap();
        note.write_to_db(&mut conn, timestamp, deck_id)
            .await
            .unwrap();
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok() {
        assert_eq!(
            find_invalid_html_tags_in_field("<h1>"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_with_space() {
        assert_eq!(
            find_invalid_html_tags_in_field(" <h1> "),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_multiple() {
        assert_eq!(
            find_invalid_html_tags_in_field("<h1>test</h1>"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_br() {
        assert_eq!(
            find_invalid_html_tags_in_field("<br>"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_br2() {
        assert_eq!(
            find_invalid_html_tags_in_field("<br/>"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_br3() {
        assert_eq!(
            find_invalid_html_tags_in_field("<br />"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ok_attrs() {
        assert_eq!(
            find_invalid_html_tags_in_field(r#"<h1 style="color: red">STOP</h1>"#),
            Vec::<String>::new()
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ng_empty() {
        assert_eq!(
            find_invalid_html_tags_in_field(" hello <> goodbye"),
            vec!["<>"]
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ng_empty_space() {
        assert_eq!(
            find_invalid_html_tags_in_field(" hello < > goodbye"),
            vec!["< >"]
        );
    }

    #[test]
    fn find_invalid_html_tags_in_field_ng_invalid_characters() {
        assert_eq!(find_invalid_html_tags_in_field("<@h1>"), vec!["<@h1>"]);
    }

    #[test]
    fn find_invalid_html_tags_in_field_ng_invalid_characters_end() {
        assert_eq!(find_invalid_html_tags_in_field("<h1@>"), vec!["<h1@>"]);
    }

    #[test]
    fn option_builder() -> anyhow::Result<()> {
        // Make sure we can call the different builder-style methods on Note.
        // Doesn't actually verify any behavior though.
        let model = Model::new(
            1234,
            "model",
            vec![Field::new("a"), Field::new("b")],
            vec![Template::new("template")],
        );
        let _note = Note::new(model, vec!["a", "b"])?
            .guid("1234")
            .tags(["tag_a"])
            .with_tag("tag_b")
            .sort_field(true);

        Ok(())
    }
}

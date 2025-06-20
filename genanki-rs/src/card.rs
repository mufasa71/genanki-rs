use anyhow::Result;
use sqlx::SqliteConnection;

#[derive(Clone)]
pub struct Card {
    pub ord: i64,
    pub suspend: bool,
}

impl Card {
    pub fn new(ord: i64, suspend: bool) -> Self {
        Self { ord, suspend }
    }
    #[allow(dead_code)]
    pub fn ord(&self) -> i64 {
        self.ord
    }
    pub async fn write_to_db(
        &self,
        conn: &mut SqliteConnection,
        timestamp: f64,
        deck_id: i64,
        note_id: usize,
    ) -> Result<()> {
        let queue = if self.suspend { -1 } else { 0 };
        let note_id = note_id as i64;
        let timestamp = timestamp as i64;

        sqlx::query!(
            r#"
                INSERT INTO cards (nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data)
                VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
            "#,
            note_id,          // nid
            deck_id,          // did
            self.ord,         // ord
            timestamp,        // mod
            -1,               // usn
            0,                // type (=0 for non-Cloze)
            queue,            // queue
            0,                // due
            0,                // ivl
            0,                // factor
            0,                // reps
            0,                // lapses
            0,                // left
            0,                // odue
            0,                // odid
            0,                // flags
            "",               // data
        ).execute(conn).await?;

        Ok(())
    }
}

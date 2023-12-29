use std::env;

mod db;
mod deepl;
mod google;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let command = &args[1];

  if command == "seed" {
    db::init_db();
    db::seed_db()?;
  }

  if command == "translate" {
    google::translate().await?;
  }

  if command == "synthesize" {
    google::synthesize().await?;
  }

  if command == "translate_ai" {
    deepl::translate_ai().await?;
  }
  Ok(())
}

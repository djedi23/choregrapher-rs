use crate::{choregraphy::Choregraphy, Opts};
use anyhow::{Context, Error, Result};
use chrono::SecondsFormat;
use indicatif::ProgressBar;
use mongodb::{
  bson::{doc, from_document},
  options::FindOptions,
};
use node_rs::{db::get_graph_collection, settings::Settings};
use std::fmt;
use tokio_stream::StreamExt;

impl fmt::Display for Choregraphy {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:<16} {:<8} {:<26}",
      self.id,
      self.version,
      self
        .created_at
        .to_chrono()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
    )
  }
}

pub async fn ls(_opt: &Opts, settings: &Settings) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Fetching chroregraphies");
  let collection = get_graph_collection(settings)
    .await
    .context("Accessing mongo")?;

  let mut cursor = collection
    .find(doc! {}, FindOptions::default()) // TODO sort by date desc
    .await
    .context("Can't fetch the choregraphies list.")?;
  spin.finish_and_clear();

  println!("{:<16} {:<8} {:<26}", "ID", "VERSION", "CREATED");
  while let Some(result) = cursor.next().await {
    match result {
      Ok(document) => {
        let chor: Choregraphy = from_document(document).context("Can't decode the choregraphies")?;
        println!("{}", chor);
      }
      Err(e) => return Err(e.into()),
    }
  }

  Ok::<(), Error>(())
}

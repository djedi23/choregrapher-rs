use crate::choregraphy::Choregraphy;
use crate::Opts;
use anyhow::{Context, Error, Result};
use bson::doc;
use chrono::SecondsFormat;
use futures::stream::StreamExt;
use futures_executor::LocalPool;
use indicatif::ProgressBar;
use mongodb::options::FindOptions;
use node_rs::{db::get_graph_collection, settings::Settings};
use std::fmt;

impl fmt::Display for Choregraphy {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:<16} {:<8} {:<26}",
      self.id,
      self.version,
      self.created_at.to_rfc3339_opts(SecondsFormat::Millis, true)
    )
  }
}

pub fn ls(_opt: &Opts, settings: &Settings) -> Result<()> {
  let mut executor = LocalPool::new();
  executor.run_until(async {
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
          let chor: Choregraphy =
            bson::from_document(document).context("Can't decode the choregraphies")?;
          println!("{}", chor);
        }
        Err(e) => return Err(e.into()),
      }
    }

    Ok::<(), Error>(())
  })
}

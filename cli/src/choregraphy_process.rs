use crate::{choregraphy::ChoregraphyProcessOptions, Opts};
use anyhow::{Context, Error, Result};
use chrono::SecondsFormat;
use core::fmt;
use indicatif::ProgressBar;
use mongodb::{
  bson::{doc, from_document, DateTime},
  options::FindOptions,
};
use node_rs::{db::get_collection, settings::Settings};
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Deserialize, Debug)]
pub(crate) struct Process {
  #[serde(rename(deserialize = "processId"))]
  pub(crate) process_id: String,
  pub(crate) timestamp: DateTime,
}

impl fmt::Display for Process {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:<36} {:<26}",
      self.process_id,
      self
        .timestamp
        .to_chrono()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
    )
  }
}

pub(crate) async fn process(
  process_opts: &ChoregraphyProcessOptions,
  _opt: &Opts,
  settings: Arc<Settings>,
) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Fetching processes");
  let collection = get_collection(&settings).await.context("Accessing mongo")?;
  let mut options = FindOptions::default();
  if process_opts.time {
    options.sort = Some(doc! {"timestamp": ( if process_opts.reverse {1} else {-1}) })
  } else {
    options.sort = Some(doc! {"processId": ( if process_opts.reverse {-1} else {1}) })
  }

  let mut cursor = collection
    .find(doc! {"graphId": &process_opts.graph}, options)
    .await
    .context("Can't fetch the choregraphies list.")?;
  spin.finish_and_clear();

  if _opt.verbose > 0 || process_opts.verbose {
    println!("{:<36} {:<26}", "PROCESS ID", "CREATED");
  }
  while let Some(result) = cursor.next().await {
    match result {
      Ok(document) => {
        let process: Process = from_document(document).context("Can't decode the process")?;
        if _opt.verbose > 0 || process_opts.verbose {
          println!("{process}");
        } else {
          println!("{}", process.process_id);
        }
      }
      Err(e) => return Err(e.into()),
    }
  }
  Ok::<(), Error>(())
}

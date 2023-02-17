use crate::{InjectOptions, Opts};
use anyhow::{bail, Context, Error, Result};
use indicatif::ProgressBar;
use mongodb::{
  bson::{doc, from_document},
  options::FindOneOptions,
};
use node_rs::{
  builder::GraphInternal, graph::Graph, rabbitmq::FieldAccessor, settings::Settings,
  start_process::start_process,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ValueWraper(Value);
impl FieldAccessor for ValueWraper {
  fn field_list(self) -> Vec<String> {
    dbg!("{}", self);
    todo!()
  }

  fn field(self) -> Vec<String> {
    let value: Value = self.0;
    value
      .as_object()
      .unwrap()
      .keys()
      .cloned()
      .collect::<Vec<_>>()
  }
}

pub async fn inject(inject_opts: &InjectOptions, _opt: &Opts, settings: Arc<Settings>) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Fetching chroregraphy");

  let app = node_rs::App::new_with_setting(&inject_opts.graph, settings).await;
  let document = app
    .graph_collection
    .find_one(doc! {"id":&inject_opts.graph}, FindOneOptions::default())
    .await
    .context("Can't fetch the choregraphy.")?;
  spin.finish_and_clear();

  if let Some(document) = document {
    let spin = ProgressBar::new_spinner();
    spin.enable_steady_tick(150);
    spin.set_message("Injecting data");
    let chor: Graph = from_document(document).context("Can't decode the choregraphy")?;
    let gi: GraphInternal = GraphInternal::new(chor, app.clone());
    let process_id = start_process(
      app,
      gi,
      &serde_json::from_str::<ValueWraper>(&inject_opts.data)
        .context("Can't decode the data to inject")?,
      &None,
    )
    .await;
    spin.finish_and_clear();
    println!("{process_id}");
  } else {
    bail!("Choregraphy not found: {}", &inject_opts.graph);
  }
  Ok::<(), Error>(())
}

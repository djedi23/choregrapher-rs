use crate::{InjectOptions, Opts};
use anyhow::{bail, Context, Error, Result};
use bson::doc;
use futures_executor::LocalPool;
use indicatif::ProgressBar;
use mongodb::options::FindOneOptions;
use node_rs::{
  builder::GraphInternal,
  db::get_graph_collection,
  graph::Graph,
  rabbitmq::{create_rabbit_mq, FieldAccessor},
  settings::Settings,
  start_process::start_process,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

pub fn inject(inject_opts: &InjectOptions, _opt: &Opts, settings: &Settings) -> Result<()> {
  let mut executor = LocalPool::new();
  let spawner = executor.spawner();
  executor.run_until(async {
    let spin = ProgressBar::new_spinner();
    spin.enable_steady_tick(150);
    spin.set_message("Fetching chroregraphy");
    let collection = get_graph_collection(settings)
      .await
      .context("Accessing mongo")?;

    let document = collection
      .find_one(doc! {"id":&inject_opts.graph}, FindOneOptions::default())
      .await
      .context("Can't fetch the choregraphy.")?;
    spin.finish_and_clear();

    if let Some(document) = document {
      let spin = ProgressBar::new_spinner();
      spin.enable_steady_tick(150);
      spin.set_message("Injecting data");
      let chor: Graph = bson::from_document(document).context("Can't decode the choregraphy")?;
      let (channel, channel_out) = create_rabbit_mq(&inject_opts.graph).await?;
      let gi: GraphInternal = GraphInternal::new(chor, &channel, &channel_out, &spawner);
      let process_id = start_process(
        &channel_out,
        gi,
        &serde_json::from_str::<ValueWraper>(&inject_opts.data)
          .context("Can't decode the data to inject")?,
        &None,
      )
      .await;
      spin.finish_and_clear();
      println!("{}", process_id);
    } else {
      bail!("Choregraphy not found: {}", &inject_opts.graph);
    }
    Ok::<(), Error>(())
  })
}

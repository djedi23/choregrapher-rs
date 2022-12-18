use crate::{
  choregraphy::{Choregraphy, ChoregraphyInspectOptions},
  Opts,
};
use anyhow::{bail, Context, Error, Result};
use bson::doc;
use futures_executor::LocalPool;
use indicatif::ProgressBar;
use mongodb::options::FindOneOptions;
use node_rs::{db::get_graph_collection, settings::Settings};
use serde_json::{json, to_string_pretty, Value};
use std::collections::HashMap;

pub fn inspect(
  inspect_opts: &ChoregraphyInspectOptions,
  _opt: &Opts,
  settings: &Settings,
) -> Result<()> {
  let mut executor = LocalPool::new();
  executor.run_until(async {
    let spin = ProgressBar::new_spinner();
    spin.enable_steady_tick(150);
    spin.set_message("Fetching chroregraphy");
    let collection = get_graph_collection(settings)
      .await
      .context("Accessing mongo")?;

    let document = collection
      .find_one(doc! {"id":&inspect_opts.graph}, FindOneOptions::default())
      .await
      .context("Can't fetch the choregraphy.")?;
    spin.finish_and_clear();

    if let Some(document) = document {
      //   println!("{:?}", document);
      if inspect_opts.json {
        print!("{}", to_string_pretty(&document)?);
      } else {
        let choregraphy: Choregraphy =
          bson::from_document(document).context("Can't decode the choregraphy")?;

        println!(
          "{}:{}
",
          choregraphy.id, choregraphy.version
        );
        let starting_parameters: Vec<String> = choregraphy
          .edges
          .iter()
          .filter_map(|e| {
            if e.from.node == "start" {
              Some(e.from.output.clone())
            } else {
              None
            }
          })
          .collect();
        let mut parameter_map: HashMap<String, Value> = HashMap::new();
        starting_parameters.iter().for_each(|parameter| {
          parameter_map.insert(parameter.clone(), json!([3]));
        });
        println!(
          "Start the choregraphy using this command line:
dancer inject {} '{}'",
          choregraphy.id,
          serde_json::to_string(&parameter_map)?
        );
      }
    } else {
      bail!("Choregraphy not found: {}", &inspect_opts.graph);
    }

    Ok::<(), Error>(())
  })
}

use crate::Opts;
use anyhow::{Context, Result};
use bson::{doc, Bson};
use clap::Parser;
use core::fmt;
use indicatif::ProgressBar;
use mongodb::options::FindOptions;
use node_rs::{db::get_collection, flow_message::NodeOutput, settings::Settings};
use std::fmt::Display;
use tokio_stream::StreamExt;

#[derive(Parser, Debug)]
pub(crate) struct ProcessOptions {
  #[clap(subcommand)]
  pub(crate) command: ProcessSubCommand,
}

/// Processes operations.
///
///
#[derive(Parser, Debug)]
#[clap(
  version = "0.1.0",
  author = "Valvassori M. <moise.valvassori@gmail.com>"
)]
pub(crate) enum ProcessSubCommand {
  // /// Inspect a choregraphy
  // Inspect(ProcessInspectOptions),
  /// Display process output
  #[clap(alias = "out")]
  Output(ProcessOutputOptions),
}

#[derive(Parser, Debug)]
#[clap(
  version = "0.1.0",
  author = "Valvassori M. <moise.valvassori@gmail.com>"
)]
pub struct ProcessOutputOptions {
  #[clap(long, short)]
  /// Returs all output of all node
  pub all: bool,

  #[clap(long, short)]
  /// Filter the outputs
  pub filter: Option<String>,

  /// json process display
  #[clap(short, long)]
  pub json: bool,

  /// verbose process display
  #[clap(short, long)]
  pub verbose: bool,

  /// Process Id
  pub process: String,
}

pub(crate) async fn process_options(
  pr_opts: &ProcessOptions,
  opts: &Opts,
  settings: &Settings,
) -> Result<()> {
  match &pr_opts.command {
    ProcessSubCommand::Output(options) => process_output(options, opts, settings).await,
  }
}

struct ProcessOutputVerboseDisplay {
  out: NodeOutput<Bson>,
}

impl From<NodeOutput<Bson>> for ProcessOutputVerboseDisplay {
  fn from(out: NodeOutput<Bson>) -> ProcessOutputVerboseDisplay {
    ProcessOutputVerboseDisplay { out }
  }
}

impl Display for ProcessOutputVerboseDisplay {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let out = &self.out;
    write!(
      f,
      "{:<36} {:<26}{:>8}.{:<8} {}",
      out.process_id,
      out
        .timestamp
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      out.output.node,
      out.output.output,
      out.parameter
    )
  }
}

struct ProcessOutputDisplay {
  out: NodeOutput<Bson>,
}

impl From<NodeOutput<Bson>> for ProcessOutputDisplay {
  fn from(out: NodeOutput<Bson>) -> ProcessOutputDisplay {
    ProcessOutputDisplay { out }
  }
}

impl Display for ProcessOutputDisplay {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let out = &self.out;
    write!(f, "{}", out.parameter)
  }
}

pub(crate) async fn process_output(
  out_opts: &ProcessOutputOptions,
  opts: &Opts,
  settings: &Settings,
) -> Result<()> {
  let spin = ProgressBar::new_spinner();
  spin.enable_steady_tick(150);
  spin.set_message("Fetching processes");
  let collection = get_collection(settings).await.context("Accessing mongo")?;
  let projection = if out_opts.all {
    doc! {"outputs":1, "parameters":-1}
  } else {
    doc! {"outputs":{"$slice":-1}, "parameters":-1}
  };

  let options = FindOptions::builder().projection(projection).build();

  let mut cursor = collection
    .find(doc! {"processId": &out_opts.process}, options)
    .await
    .context("Can't fetch the choregraphies list.")?;
  spin.finish_and_clear();

  if opts.verbose > 0 || out_opts.verbose {
    println!(
      "{:<36} {:<26}{:<17} OUTPUT",
      "PROCESS ID", "TIMESTAMP", "NODE"
    );
  }
  while let Some(result) = cursor.next().await {
    match result {
      Ok(document) => {
        let output_doc = document
          .get_array("outputs")
          .context("Can't find the output in process document")?;
        if out_opts.json {
          let j = if out_opts.all {
            serde_json::to_string_pretty(&output_doc)?
          } else {
            serde_json::to_string_pretty(&output_doc[0].as_document().unwrap())?
          };
          println!("{}", j);
        } else {
          for output_elt in output_doc {
            let output: NodeOutput<Bson> =
              bson::from_document(output_elt.as_document().unwrap().clone())
                .context("Can't decode the process output")?;

            if out_opts.verbose {
              println!("{}", ProcessOutputVerboseDisplay::from(output));
            } else {
              println!("{}", ProcessOutputDisplay::from(output));
            }
          }
        }
      }
      Err(e) => return Err(e.into()),
    }
  }

  Ok(())
}

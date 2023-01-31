use crate::{choregraphy_inspect::inspect, choregraphy_ls::ls, choregraphy_process::process, Opts};
use anyhow::Result;
use bson::{doc, DateTime};
use clap::Parser;
use node_rs::settings::Settings;
use serde::Deserialize;

#[derive(Parser, Debug)]
pub(crate) struct ChoregraphyOptions {
  #[clap(subcommand)]
  pub(crate) command: ChoregraphySubCommand,
}

/// Choreraphies graph operations.
///
///
#[derive(Parser, Debug)]
#[clap(
  version = "0.1.0",
  author = "Valvassori M. <moise.valvassori@gmail.com>"
)]
pub(crate) enum ChoregraphySubCommand {
  /// List the choregraphies
  Ls,
  /// Inspect a choregraphy
  Inspect(ChoregraphyInspectOptions),
  /// List the choregraphy's processes
  #[clap(alias = "ps")]
  Process(ChoregraphyProcessOptions),
}

#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "Valvassori M. <moise.valvassori@gmail.com>")]
pub struct ChoregraphyInspectOptions {
  /// Id of the graph to inspect
  pub(crate) graph: String,
  /// Display the choragraphy as json
  #[clap(long)]
  pub(crate) json: bool,
}

#[derive(Parser, Debug)]
#[clap(
  version = "0.1.0",
  author = "Valvassori M. <moise.valvassori@gmail.com>"
)]
pub struct ChoregraphyProcessOptions {
  /// Id of the graph to inspect
  pub(crate) graph: String,
  /// sort by time, newest first
  #[clap(short, long)]
  pub time: bool,
  /// reverse order while sorting
  #[clap(short, long)]
  pub reverse: bool,
  /// verbose process display
  #[clap(short, long)]
  pub verbose: bool,
}

pub(crate) async fn choregraphy_options(
  ch_opts: &ChoregraphyOptions,
  opts: &Opts,
  settings: &Settings,
) -> Result<()> {
  match &ch_opts.command {
    ChoregraphySubCommand::Ls => ls(opts, settings).await,
    ChoregraphySubCommand::Inspect(inspect_opts) => inspect(inspect_opts, opts, settings).await,
    ChoregraphySubCommand::Process(_process_opts) => process(_process_opts, opts, settings).await,
  }
}

////////////////////////////////////////////////////////////

#[derive(Deserialize, Debug)]
pub(crate) struct Output {
  pub(crate) node: String,
  pub(crate) output: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Input {
  // node: String,
  // input: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Edge {
  pub from: Output,
  //  pub to: Input,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Choregraphy {
  pub(crate) id: String,
  pub(crate) version: String,
  #[serde(rename(deserialize = "createdAt"))]
  pub(crate) created_at: DateTime,
  // #[serde(rename(deserialize = "modifiedAt"))]
  // modified_at: DateTime,

  // #[serde(skip)]
  // startingParameters: String,
  // #[serde(skip)]
  // nodes: String,
  //  #[serde(skip)]
  pub(crate) edges: Vec<Edge>,
}

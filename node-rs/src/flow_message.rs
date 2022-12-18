use crate::{context::FlowContext, graph::OutputRef};
use bson::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowMessage<T> {
  #[serde(rename = "processId")]
  pub process_id: String,
  pub parameter: T,
  pub context: Option<FlowContext>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialFlowMessage {
  #[serde(rename = "processId")]
  pub process_id: String,
  pub parameter: Value,
  pub context: Option<FlowContext>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeOutput<T> {
  #[serde(rename = "processId")]
  pub process_id: String,
  pub parameter: T,
  // context
  pub output: OutputRef,
  pub timestamp: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialNodeOutput {
  #[serde(rename = "processId")]
  pub process_id: String,
  // context
  pub output: OutputRef,
  pub timestamp: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowRunDB<T> {
  #[serde(rename = "processId")]
  pub process_id: String,
  #[serde(rename = "graphId")]
  pub graph_id: String,
  pub outputs: Vec<NodeOutput<T>>,
}

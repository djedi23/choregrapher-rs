//! # Choragraphy builder
//!
use crate::{
  graph::{Graph, InputRef, Node, Relation},
  output_processor::{DefaultOutputProcessor, OutputProcessing},
  rabbitmq::{create_consumer, FieldAccessor},
  settings::Settings,
  App, NodeAction,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tracing::instrument;

/// Hold the internal information to process a graph
pub struct GraphInternal {
  /// graph id
  pub(crate) id: String,
  /// A copy of the nodes
  nodes: HashMap<String, Arc<Node>>,
  /// Dict of node input from node output. Used when you send message
  /// to node input.
  pub(crate) destination_map: Arc<HashMap<String, Vec<InputRef>>>,
  origin_map: Arc<HashMap<String, Vec<Relation>>>,

  /// Copy for internal use
  app: Arc<App>,
}

impl GraphInternal {
  /// Create a new graph internals from a graph description
  /// param graph the graph description to internalize
  pub fn new(graph: Graph, app: Arc<App>) -> GraphInternal {
    let id = graph.id;
    let mut nodes = HashMap::new();
    for node in graph.nodes {
      let nodecopy = Arc::new(Node {
        id: node.id.to_string(),
        name: node.name.to_string(),
        input: node.input.clone(),
        output: node.output.clone(),
      });
      nodes.insert(node.id.to_string(), nodecopy);
    }
    let settings = Settings::new().unwrap();
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    let mut origin_map: HashMap<String, Vec<Relation>> = HashMap::new();
    for relation in graph.edges {
      let destination_map_key = String::from(&relation.from.node) + "_" + &relation.from.output;
      match destination_map.get_mut(&destination_map_key) {
        None => {
          let mut vec_inputs = Vec::new();
          {
            let mutvec_inputs = &mut vec_inputs;
            mutvec_inputs.push(relation.to.clone());
          }
          destination_map.insert(destination_map_key, vec_inputs);
        }
        Some(relations) => (relations).push(relation.to.clone()),
      }

      let origin_map_key =
        settings.rabbitmq.routing_key.clone() + "." + &relation.to.node + "_" + &relation.to.input;
      match origin_map.get_mut(&origin_map_key) {
        None => {
          let mut vec_outputs = Vec::new();
          {
            let mutvec_outputs = &mut vec_outputs;
            mutvec_outputs.push(relation);
          }
          origin_map.insert(origin_map_key, vec_outputs);
        }
        Some(relations) => (relations).push(relation),
      }
    }

    GraphInternal {
      id,
      nodes,
      destination_map: Arc::new(destination_map),
      origin_map: Arc::new(origin_map),
      app,
    }
  }

  /// Get a node by its id
  fn node(&self, id: &str) -> Option<&Arc<Node>> {
    self.nodes.get(id)
  }

  #[instrument(level = "trace", skip(self, action, output_processor))]
  pub async fn register_node_action<
    I: 'static + Send,
    R: 'static + Send + Sync,
    O: 'static + Send + Sync,
  >(
    &self,
    node_id: &str,
    action: Arc<dyn NodeAction<INPUT = I, OUTPUT = R>>,
    output_processor: Option<
      Arc<impl OutputProcessing<INPUT = R, OUTPUT = O> + 'static + Send + Sync>,
    >,
  ) where
    I: Debug + DeserializeOwned,
    R: Debug + Serialize + FieldAccessor + Clone,
    O: Debug + Serialize + FieldAccessor + Clone + From<R>,
  {
    let node = self.node(node_id);
    if let Some(node) = node {
      let op: Arc<dyn OutputProcessing<INPUT = R, OUTPUT = O> + 'static + Sync + Send> =
        match output_processor {
          Some(output_processor) => output_processor,
          None => Arc::new(DefaultOutputProcessor::default()),
        };

      create_consumer(
        self.app.clone(),
        self.destination_map.clone(),
        self.origin_map.clone(),
        self.id.clone(),
        node.clone(),
        action,
        op,
      )
      .await;
    }
  }
}

#[cfg(test)]
mod tests {
  //  use super::*;
  use crate::graph::*;

  // #SPC-graph-internal.tst-id
  #[test]
  fn graph_internal_new() {
    let _graph = Graph {
      id: String::from("fact"),
      nodes: vec![Node {
        id: "fact".to_string(),
        name: String::from("Factorial"),
        input: vec!["i".to_string()],
        output: vec!["o".to_string(), "r".to_string()],
      }],
      edges: vec![
        Relation {
          from: OutputRef {
            node: "start".to_string(),
            output: "o".to_string(),
          },
          to: InputRef {
            node: "factorial".to_string(),
            input: "i".to_string(),
          },
        },
        Relation {
          from: OutputRef {
            node: "fact".to_string(),
            output: "o".to_string(),
          },
          to: InputRef {
            node: "factorial".to_string(),
            input: "i".to_string(),
          },
        },
      ],
    };

    // let gi = GraphInternal::new(graph.clone());
    // match gi.node(&"fact".to_string()) {
    //   Some(node) => assert_eq!(node.id, "fact"),
    //   None => assert!(false, "No 'fact' node found"),
    // }
    // match gi.node(&"unknowed".to_string()) {
    //   Some(_) => assert!(false, "'unknowed' node shouldn't be found"),
    //   None => assert!(true),
    // }

    // match gi.output_destination(&graph.edges[0].from) {
    //   Some(vec_of_input) => {
    //     assert_eq!(vec_of_input[0].node, "factorial");
    //     assert_eq!(vec_of_input[0].input, "i")
    //   }
    //   None => assert!(false, "Destinations should be found but none"),
    // }
  }
}

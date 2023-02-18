//! # Computing Graph Description
//!
//! `graph` is a collection of structure describing the computing graph.
//! ```json
//! {
//!  "id": "fact",
//!  "nodes": [
//!   {
//!    "id": "fact",
//!    "input": [ "i" ],
//!    "output": [ "o", "r" ]
//!   }
//!  ],
//!  "edges": [
//!   {
//!    "from": { "node": "start", "output": "i" },
//!    "to": {   "node": "fact",  "input": "i"  }
//!   },
//!   {
//!    "from": {  "node": "fact", "output": "o" },
//!    "to": {    "node": "fact", "input": "i"  }
//!   }
//!  ]
//! }
//! ```

use serde::{Deserialize, Serialize};

// #SPC-graph.graph
#[derive(Serialize, Deserialize, Debug, Clone)]
/// The main graph structure
pub struct Graph {
  pub id: String,
  pub nodes: Vec<Node>,
  // #SPC-graph.edge
  pub edges: Vec<Relation>,
}

// #SPC-graph.node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Node {
  pub id: String,
  pub name: String,
  pub input: Vec<String>,
  pub output: Vec<String>,
}

impl Node {
  pub fn id(id: String) -> Node {
    Node {
      id,
      name: "".into(),
      input: Vec::default(),
      output: Vec::default(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputRef {
  pub node: String,
  pub input: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OutputRef {
  pub node: String,
  pub output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relation {
  pub from: OutputRef,
  pub to: InputRef,
}

#[cfg(test)]
mod tests {
  use super::*;

  // #SPC-graph.tst-id
  #[test]
  fn graph_creation_without_nodes() {
    let g = Graph {
      id: "myid".to_string(),
      nodes: vec![],
      edges: vec![],
    };
    assert_eq!(g.id, "myid");
  }

  #[test]
  fn node_creation() {
    let n = Node {
      id: "myid".to_string(),
      name: String::from("myname"),
      input: vec![],
      output: vec![],
    };
    assert_eq!(n.id, "myid");
    assert_eq!(n.name, "myname");
  }

  #[test]
  fn graph_creation() {
    let g = Graph {
      id: String::from("myid"),
      nodes: vec![
        Node {
          id: "a".to_string(),
          name: String::from("myname 1"),
          input: vec![],
          output: vec![],
        },
        Node {
          id: "b".to_string(),
          name: String::from("myname 2"),
          input: vec![],
          output: vec![],
        },
      ],
      edges: vec![Relation {
        from: OutputRef {
          node: "a".to_string(),
          output: "i".to_string(),
        },
        to: InputRef {
          node: "b".to_string(),
          input: "i".to_string(),
        },
      }],
    };
    assert_eq!(g.id, "myid");
    assert_eq!(g.nodes[0].id, "a");
    assert_eq!(g.nodes[1].id, "b");
    assert_eq!(g.edges[0].from.node, "a");
  }

  #[test]
  fn graph_creation_without_nodes_from_json() {
    let g: Graph = serde_json::from_str("{\"id\":\"myid\",\"nodes\":[],\"edges\":[]}").unwrap();
    assert_eq!(g.id, "myid");
  }

  #[test]
  fn graph_creation_from_json() {
    let g: Graph = serde_json::from_str(
      "{\"id\":\"myid\",\"nodes\":[{\"id\":\"a\",\"name\":\"na\",\"input\":[\"a\"],\"output\":[\"b\",\"c\"]},{\"id\":\"b\",\"name\":\"nb\",\"input\":[\"a\"],\"output\":[\"b\",\"c\"]}],\"edges\":[]}",
    )
    .unwrap();
    assert_eq!(g.id, "myid");
    assert_eq!(g.nodes[0].id, "a");
    assert_eq!(g.nodes[0].input[0], "a");
    assert_eq!(g.nodes[0].output[0], "b");
    assert_eq!(g.nodes[0].output[1], "c");
    assert_eq!(g.nodes[1].id, "b");
  }
}

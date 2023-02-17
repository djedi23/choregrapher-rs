use crate::{
  graph::{InputRef, Node, OutputRef},
  rabbitmq::FieldAccessor,
  settings::Settings,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tracing::trace;
use yansi::Paint;

#[derive(Debug)]
pub struct Route {
  pub topic: String,
  pub payload: Value,
}

/// Returns the inputs to be routed by an o{ topic: (), payload: ()}topic: (), payload: ()}put
fn output_destination(
  destination_map: &'_ HashMap<String, Vec<InputRef>>,
  output_ref: OutputRef,
) -> Option<&'_ Vec<InputRef>> {
  let destination_key = String::from(&output_ref.node) + "_" + &output_ref.output;
  destination_map.get(&destination_key)
}

// partof: #SPC-processing.routingOutput
/// Route an output to none, one or more inputs.
#[tracing::instrument(level = "debug")]
pub fn routing<R: Debug + Clone + FieldAccessor + Serialize>(
  destination_map: &HashMap<String, Vec<InputRef>>,
  node: &Node,
  result: &R,
  settings: Arc<Settings>,
) -> Vec<Route> {
  result
    .clone()
    .field()
    .iter()
    .flat_map(|output_field| {
      let output_ref = OutputRef {
        node: node.id.clone(),
        output: output_field.clone(),
      };

      match output_destination(destination_map, output_ref) {
        Some(routes) => {
          let routes = routes
            .iter()
            .map(|ir| {
              // The trick is here: I convert R to serde_json::Value. Then
              // I can build the payload using the first field (the output) to the input
              // field..
              let payload = serde_json::to_value(result).unwrap();
              match payload {
                Value::Object(o) => {
                  let payload = json!({ir.input.to_string() : o.get(output_field).unwrap().clone()});
                  Route {
                    topic: settings.rabbitmq.routing_key.clone() + "." + &ir.node + "_" + &ir.input,
                    payload,
                  }
                }
                _ => panic!("Object not found"),
              }
            })
            .collect::<Vec<Route>>();

          trace!(
            "routing {:?} {:?}",
            Paint::blue(node),
            serde_json::to_string_pretty(result),
            //        Paint::green(routes.clone())
          );

          routes
        }
        None => vec![],
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use field_accessor_derive::FieldAccessor;
  use serde::{Deserialize, Serialize};
  use serde_json::json;
  use std::ops::{Add, Mul, Sub};

  // #SPC-processing.tst-retrieve_output_destination0
  #[test]
  fn output_destination_found_none() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_oo".to_string(),
      vec![InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      }],
    );
    let output_ref = OutputRef {
      node: "fact".to_string(),
      output: "o".to_string(),
    };

    let result = output_destination(&destination_map, output_ref);

    match result {
      Some(_) => unreachable!("Shouldn't have found something"),
      None => "Should return result", // assert!(true, "Should return result"),
    };
  }

  // #SPC-processing.tst-retrieve_output_destination1
  #[test]
  fn output_destination_found_one() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      }],
    );
    let output_ref = OutputRef {
      node: "fact".to_string(),
      output: "o".to_string(),
    };
    let expected = InputRef {
      node: "fact".to_string(),
      input: "i".to_string(),
    };

    let result = output_destination(&destination_map, output_ref);

    match result {
      Some(result_) => {
        assert_eq!(result_.len(), 1);
        assert_eq!(result_[0], expected);
      }
      None => unreachable!("Should return result"),
    }
  }

  // #SPC-processing.tst-retrieve_output_destination2
  #[test]
  fn output_destination_found_two() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![
        InputRef {
          node: "fact".to_string(),
          input: "i".to_string(),
        },
        InputRef {
          node: "fact2".to_string(),
          input: "i".to_string(),
        },
      ],
    );
    let output_ref = OutputRef {
      node: "fact".to_string(),
      output: "o".to_string(),
    };
    let expected = InputRef {
      node: "fact".to_string(),
      input: "i".to_string(),
    };
    let expected2 = InputRef {
      node: "fact2".to_string(),
      input: "i".to_string(),
    };

    let result = output_destination(&destination_map, output_ref);

    match result {
      Some(result_) => {
        assert_eq!(result_.len(), 2, "return lenght != 2");
        assert_eq!(result_[0], expected);
        assert_eq!(result_[1], expected2);
      }
      None => unreachable!("Should return result"),
    }
  }

  // #SPC-processing.tst-routing_result_to_input
  #[test]
  fn routing_result_to_input() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      }],
    );

    let node = Node {
      id: "fact".to_string(),
      name: String::from("Factorial"),
      input: vec!["i".to_string()],
      output: vec!["o".to_string(), "r".to_string()],
    };

    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum Fact<T: Sub + Mul> {
      #[serde(rename = "n")]
      N(T, T),
      #[serde(rename = "fact")]
      Fact(T),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
    enum FactResult<T: Sub + Mul> {
      #[serde(rename = "o")]
      O(Fact<T>),
      #[serde(rename = "r")]
      R(T),
    }
    let result = FactResult::O(Fact::N(10, 15));
    let routes = routing(
      &destination_map,
      &node,
      &result,
      Arc::new(Settings::default()),
    );

    assert_eq!(routes.len(), 1);
    let route = &routes[0];
    assert_eq!(route.topic, "flow.fact_i");
    assert_eq!(route.payload, json!({"i":{"n":[10,15]}}));
  }

  // #SPC-processing.tst-routing_result_to_2_inputs
  #[test]
  fn routing_result_to_2_inputs() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![
        InputRef {
          node: "fact".to_string(),
          input: "i".to_string(),
        },
        InputRef {
          node: "fact2".to_string(),
          input: "j".to_string(),
        },
      ],
    );

    let node = Node {
      id: "fact".to_string(),
      name: String::from("Factorial"),
      input: vec!["i".to_string()],
      output: vec!["o".to_string(), "r".to_string()],
    };

    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum Fact<T: Sub + Mul> {
      #[serde(rename = "n")]
      N(T, T),
      #[serde(rename = "fact")]
      Fact(T),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
    enum FactResult<T: Sub + Mul> {
      #[serde(rename = "o")]
      O(Fact<T>),
      #[serde(rename = "r")]
      R(T),
    }
    let result = FactResult::O(Fact::N(10, 15));
    let routes = routing(
      &destination_map,
      &node,
      &result,
      Arc::new(Settings::default()),
    );

    assert_eq!(routes.len(), 2);
    let route = &routes[0];
    assert_eq!(route.topic, "flow.fact_i");
    assert_eq!(route.payload, json!({"i":{"n":[10,15]}}));
    let route = &routes[1];
    assert_eq!(route.topic, "flow.fact2_j");
    assert_eq!(route.payload, json!({"j":{"n":[10,15]}}));
  }

  // #SPC-processing.tst-routing_result_to_no_input
  #[test]
  fn routing_result_to_no_input() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![
        InputRef {
          node: "fact".to_string(),
          input: "i".to_string(),
        },
        InputRef {
          node: "fact2".to_string(),
          input: "j".to_string(),
        },
      ],
    );

    let node = Node {
      id: "fact".to_string(),
      name: String::from("Factorial"),
      input: vec!["i".to_string()],
      output: vec!["o".to_string(), "r".to_string()],
    };

    #[derive(Serialize, Deserialize, Debug, Clone)]
    enum Fact<T: Sub + Mul> {
      #[serde(rename = "n")]
      N(T, T),
      #[serde(rename = "fact")]
      Fact(T),
    }

    #[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
    enum FactResult<T: Sub + Mul> {
      #[serde(rename = "o")]
      O(Fact<T>),
      #[serde(rename = "r")]
      R(T),
    }
    let result = FactResult::R(2);
    let routes = routing(
      &destination_map,
      &node,
      &result,
      Arc::new(Settings::default()),
    );

    assert_eq!(routes.len(), 0, "No route should be returned");
  }

  // #SPC-processing.tst-routing_multiple_output
  #[test]
  fn routing_result_from_multiple_oputput_to_input() {
    let mut destination_map: HashMap<String, Vec<InputRef>> = HashMap::new();
    destination_map.insert(
      "fact_o".to_string(),
      vec![InputRef {
        node: "fact".to_string(),
        input: "i".to_string(),
      }],
    );
    destination_map.insert(
      "fact_o2".to_string(),
      vec![InputRef {
        node: "fact".to_string(),
        input: "i2".to_string(),
      }],
    );

    let node = Node {
      id: "fact".to_string(),
      name: String::from("Factorial"),
      input: vec!["i".to_string(), "i2".to_string()],
      output: vec!["o".to_string(), "o2".to_string()],
    };

    #[derive(Serialize, Deserialize, Debug, Clone)] //FIXME  , FieldAccessor
    struct StartOutput<T: Add> {
      o: T,
      o2: T,
    }
    impl<T> FieldAccessor for StartOutput<T>
    where
      T: Add,
    {
      fn field_list(self) -> Vec<String> {
        vec!["o".into(), "o2".into()]
      }
      fn field(self) -> Vec<String> {
        vec!["o".into(), "o2".into()]
      }
    }

    let result = StartOutput { o: 2, o2: 5 };
    let routes = routing(
      &destination_map,
      &node,
      &result,
      Arc::new(Settings::default()),
    );

    assert_eq!(routes.len(), 2);
    let route = &routes[0];
    assert_eq!(route.topic, "flow.fact_i");
    assert_eq!(route.payload, json!({"i":2}));

    let route = &routes[1];
    assert_eq!(route.topic, "flow.fact_i2");
    assert_eq!(route.payload, json!({"i2":5}));
  }
}

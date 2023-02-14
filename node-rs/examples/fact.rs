use field_accessor_derive::FieldAccessor;
use node_rs::{
  builder::GraphInternal,
  context::Context,
  graph::{Graph, InputRef, Node, OutputRef, Relation},
  main_error::MainResult,
  output_processor::DefaultOutputProcessor,
  rabbitmq::{create_rabbit_mq, FieldAccessor},
  settings::Settings,
  start_process::start_process,
};
use serde::{Deserialize, Serialize};
use std::{
  ops::{Mul, Sub},
  sync::Arc,
};
use tracing::{debug, info};
use yansi::Paint;

// curl -X DELETE -u admin:admin 'http://localhost:15672/api/queues/%2F/flow_queue_fact/contents'

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum Fact<T: Sub + Mul> {
  #[serde(rename = "n")]
  N(T, T),
  // #[serde(rename = "fact")]
  // Fact(T),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FactInput<T: Sub + Mul> {
  i: Fact<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum FactResult<T: Sub + Mul> {
  #[serde(rename = "o")]
  O(Fact<T>),
  #[serde(rename = "r")]
  R(T),
}

#[tokio::main]
async fn main() -> MainResult<()> {
  node_rs::tracing::init();
  let settings = Settings::new().unwrap();
  info!("{:?}", settings);

  let graph = Graph {
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
          node: "fact".to_string(),
          input: "i".to_string(),
        },
      },
      Relation {
        from: OutputRef {
          node: "fact".to_string(),
          output: "o".to_string(),
        },
        to: InputRef {
          node: "fact".to_string(),
          input: "i".to_string(),
        },
      },
    ],
  };

  let (channel, channel_out) = create_rabbit_mq(&graph.id).await?;
  let gi: GraphInternal = GraphInternal::new(graph.clone(), &channel, &channel_out);

  #[tracing::instrument(level = "debug")]
  async fn fact_action(
    data: FactInput<u64>,
    _context: Arc<Context>,
  ) -> (Option<FactResult<u64>>, Arc<Context>) {
    debug!("fact action {:?}", Paint::blue(&data));
    match data.i {
      Fact::N(1, r) => {
        info!("Fact: {:?}", Paint::green(&r).bold());
        (Some(FactResult::R(r)), _context)
      }
      Fact::N(n, r) => (Some(FactResult::O(Fact::N(n - 1, r * n))), _context),
      //      Fact::Fact(_) => (None, _context),
    }
  }
  gi.register_node_action(
    "fact",
    fact_action,
    None::<Arc<DefaultOutputProcessor<FactResult<u64>, FactResult<u64>>>>,
  )
  .await;

  let init = FactResult::O(Fact::N(20, 1));
  start_process(channel_out.clone(), gi, &init, &None).await;

  loop {
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
}

use async_trait::async_trait;
use field_accessor_derive::FieldAccessor;
use node_rs::{
  builder::GraphInternal,
  context::Context,
  graph::{Graph, InputRef, Node, OutputRef, Relation},
  main_error::MainResult,
  node,
  output_processor::DefaultOutputProcessor,
  rabbitmq::FieldAccessor,
  relations,
  start_process::start_process,
  App, NodeAction,
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

#[derive(Debug)]
struct FactAction;
#[async_trait]
impl NodeAction for FactAction {
  type INPUT = FactInput<u64>;
  type OUTPUT = FactResult<u64>;

  #[tracing::instrument(name = "Fact action", level = "debug")]
  async fn action(
    &self,
    input: Self::INPUT,
    context: Arc<Context>,
  ) -> (Option<Self::OUTPUT>, Arc<Context>) {
    debug!("fact action {:?}", Paint::blue(&input));
    match input.i {
      Fact::N(1, r) => {
        info!("Fact: {:?}", Paint::green(&r).bold());
        (Some(FactResult::R(r)), context)
      }
      Fact::N(n, r) => (Some(FactResult::O(Fact::N(n - 1, r * n))), context),
      //      Fact::Fact(_) => (None, _context),
    }
  }
}

#[tokio::main]
async fn main() -> MainResult<()> {
  node_rs::tracing::init();
  let graph = Graph {
    id: String::from("fact"),
    nodes: vec![node!((fact:Factorial) (i)->(o,r))],
    edges: relations![start(o)->(i)fact,
		      fact(o)->(i)fact],
  };

  let app = App::new(&graph.id).await;
  info!("{:?}", app.settings);
  let gi: GraphInternal = GraphInternal::new(graph.clone(), app.clone());
  gi.register_node_action(
    "fact",
    Arc::new(FactAction),
    None::<Arc<DefaultOutputProcessor<FactResult<u64>, FactResult<u64>>>>,
  )
  .await;

  let init = FactResult::O(Fact::N(20, 1));
  start_process(app, gi, &init, &None).await;

  loop {
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
}

use async_trait::async_trait;
use field_accessor_derive::FieldAccessor;
use node_rs::{
  builder::GraphInternal,
  context::Context,
  graph::{Graph, InputRef, Node, OutputRef, Relation},
  main_error::MainResult,
  output_processor::DefaultOutputProcessor,
  rabbitmq::FieldAccessor,
  start_process::start_process,
  App, NodeAction,
};
use serde::{Deserialize, Serialize};
use std::{ops::Add, sync::Arc, time::Duration};
use tracing::{debug, info};
use yansi::Paint;

// curl -X DELETE -u admin:admin 'http://localhost:15672/api/queues/%2F/flow_queue_proxy1/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_proxy/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_sum/contents'

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum ProxyData<T: Add> {
  #[serde(rename = "i")]
  I(T),
}

#[derive(Debug)]
struct ProxyAction;
#[async_trait]
impl NodeAction for ProxyAction {
  type INPUT = ProxyData<u64>;
  type OUTPUT = ProxyData<u64>;

  #[tracing::instrument(name = "Proxy Action", level = "debug")]
  async fn action(
    &self,
    input: Self::INPUT,
    context: Arc<Context>,
  ) -> (Option<Self::OUTPUT>, Arc<Context>) {
    debug!("proxy action {:?}", Paint::cyan(&input));
    match input {
      ProxyData::I(i) => (Some(ProxyData::I(i)), context),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SumInput<T: Add> {
  i1: Option<T>,
  i2: Option<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct SumOutput<T: Add> {
  o: T,
}

#[derive(Debug)]
struct SumAction;
#[async_trait]
impl NodeAction for SumAction {
  type INPUT = SumInput<u64>;
  type OUTPUT = SumOutput<u64>;

  #[tracing::instrument(name = "Sum Action", level = "debug")]
  async fn action(
    &self,
    input: Self::INPUT,
    context: Arc<Context>,
  ) -> (Option<Self::OUTPUT>, Arc<Context>) {
    debug!("sum action {:?}", input);
    (
      Some(SumOutput {
        o: input.i1.unwrap_or_default() + input.i2.unwrap_or_default(),
      }),
      context,
    )
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct StartOutput<T: Add> {
  i1: T,
  i2: T,
}

#[tokio::main]
async fn main() -> MainResult<()> {
  node_rs::tracing::init();

  let graph = Graph {
    id: String::from("summer"),
    nodes: vec![
      Node {
        id: "sum".to_string(),
        name: String::from("Sum"),
        input: vec!["i1".to_string(), "i2".to_string()],
        output: vec!["o".to_string()],
      },
      Node {
        id: "proxy".to_string(),
        name: "Proxy integer".to_string(),
        input: vec!["i".to_string()],
        output: vec!["i".to_string()],
      },
      Node {
        id: "proxy1".to_string(),
        name: "Proxy integer 1".to_string(),
        input: vec!["i".to_string()],
        output: vec!["i".to_string()],
      },
    ],
    edges: vec![
      Relation {
        from: OutputRef {
          node: "start".to_string(),
          output: "i1".to_string(),
        },
        to: InputRef {
          node: "proxy".to_string(),
          input: "i".to_string(),
        },
      },
      Relation {
        from: OutputRef {
          node: "start".to_string(),
          output: "i2".to_string(),
        },
        to: InputRef {
          node: "proxy1".to_string(),
          input: "i".to_string(),
        },
      },
      Relation {
        from: OutputRef {
          node: "proxy".to_string(),
          output: "i".to_string(),
        },
        to: InputRef {
          node: "sum".to_string(),
          input: "i1".to_string(),
        },
      },
      Relation {
        from: OutputRef {
          node: "proxy1".to_string(),
          output: "i".to_string(),
        },
        to: InputRef {
          node: "sum".to_string(),
          input: "i2".to_string(),
        },
      },
    ],
  };

  let app = App::new(&graph.id).await;
  info!("{:?}", app.settings);
  let gi: GraphInternal = GraphInternal::new(graph.clone(), app.clone());

  gi.register_node_action(
    "sum",
    Arc::new(SumAction),
    None::<Arc<DefaultOutputProcessor<SumOutput<u64>, SumOutput<u64>>>>,
  )
  .await;

  gi.register_node_action(
    "proxy",
    Arc::new(ProxyAction),
    None::<Arc<DefaultOutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
  )
  .await;
  gi.register_node_action(
    "proxy1",
    Arc::new(ProxyAction),
    None::<Arc<DefaultOutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
  )
  .await;

  let init = StartOutput { i1: 2, i2: 3 };
  start_process(app, gi, &init, &None).await;

  loop {
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

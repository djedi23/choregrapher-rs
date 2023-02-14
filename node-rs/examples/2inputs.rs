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
use std::{ops::Add, sync::Arc, time::Duration};
use tracing::{debug, info};
use yansi::Paint;

// curl -X DELETE -u admin:admin 'http://localhost:15672/api/queues/%2F/flow_queue_proxy1/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_proxy/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_sum/contents'

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum ProxyData<T: Add> {
  #[serde(rename = "i")]
  I(T),
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

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct StartOutput<T: Add> {
  i1: T,
  i2: T,
}

#[tokio::main]
async fn main() -> MainResult<()> {
  node_rs::tracing::init();
  let settings = Settings::new().unwrap();
  info!("{:?}", settings);

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

  let (channel, channel_out) = create_rabbit_mq(&graph.id).await?;

  let gi: GraphInternal = GraphInternal::new(graph.clone(), &channel, &channel_out);

  #[tracing::instrument(level = "debug")]
  async fn sum_action(
    data: SumInput<u64>,
    _context: Arc<Context>,
  ) -> (Option<SumOutput<u64>>, Arc<Context>) {
    debug!("sum action {:?}", data);
    (
      Some(SumOutput {
        o: data.i1.unwrap_or_default() + data.i2.unwrap_or_default(),
      }),
      _context,
    )
  }

  gi.register_node_action(
    "sum",
    sum_action,
    None::<Arc<DefaultOutputProcessor<SumOutput<u64>, SumOutput<u64>>>>,
  )
  .await;

  #[tracing::instrument(level = "debug")]
  async fn proxy_action(
    data: ProxyData<u64>,
    _context: Arc<Context>,
  ) -> (Option<ProxyData<u64>>, Arc<Context>) {
    debug!("proxy action {:?}", Paint::cyan(&data));
    match data {
      ProxyData::I(i) => (Some(ProxyData::I(i)), _context),
    }
  }
  gi.register_node_action(
    "proxy",
    proxy_action,
    None::<Arc<DefaultOutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
  )
  .await;
  gi.register_node_action(
    "proxy1",
    proxy_action,
    None::<Arc<DefaultOutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
  )
  .await;

  let init = StartOutput { i1: 2, i2: 3 };
  start_process(channel_out.clone(), gi, &init, &None).await;

  loop {
    tokio::time::sleep(Duration::from_millis(1000)).await;
  }
}

use field_accessor_derive::FieldAccessor;
use futures_executor::LocalPool;
use log::{debug, info};
use node_rs::{
  builder::GraphInternal,
  context::Context,
  graph::{Graph, InputRef, Node, OutputRef, Relation},
  main_error::MainResult,
  output_processor::OutputProcessor,
  rabbitmq::{create_rabbit_mq, FieldAccessor},
  settings::Settings,
  start_process::start_process,
};
use serde::{Deserialize, Serialize};
use std::{ops::Add, rc::Rc, time::Duration};
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

fn main() -> MainResult<()> {
  pretty_env_logger::init_timed();
  let settings = Settings::new().unwrap();
  info!("{:?}", settings);

  let mut executor = LocalPool::new();
  let spawner = executor.spawner();

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

  executor.run_until(async {
    let (channel, channel_out) = create_rabbit_mq(&graph.id).await?;

    let gi: GraphInternal = GraphInternal::new(graph.clone(), &channel, &channel_out, &spawner);

    async fn sum_action(
      data: SumInput<u64>,
      _context: Rc<Context>,
    ) -> (Option<SumOutput<u64>>, Rc<Context>) {
      debug!("sum action {:?}", data);
      (
        Some(SumOutput {
          o: data.i1.unwrap_or_default() + data.i2.unwrap_or_default(),
        }),
        _context,
      )
    }

    gi.register_node_action(
      &"sum".to_string(),
      sum_action,
      None::<Box<OutputProcessor<SumOutput<u64>, SumOutput<u64>>>>,
    )
    .await;

    async fn proxy_action(
      data: ProxyData<u64>,
      _context: Rc<Context>,
    ) -> (Option<ProxyData<u64>>, Rc<Context>) {
      debug!("proxy action {:?}", Paint::cyan(&data));
      match data {
        ProxyData::I(i) => (Some(ProxyData::I(i)), _context),
      }
    }
    gi.register_node_action(
      &"proxy".to_string(),
      proxy_action,
      None::<Box<OutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
    )
    .await;
    gi.register_node_action(
      &"proxy1".to_string(),
      proxy_action,
      None::<Box<OutputProcessor<ProxyData<u64>, ProxyData<u64>>>>,
    )
    .await;

    let init = StartOutput { i1: 2, i2: 3 };
    start_process(&channel_out, gi, &init, &None).await;

    loop {
      async_std::task::sleep(Duration::from_millis(1000)).await;
    }
  })
}

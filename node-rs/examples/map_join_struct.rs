use field_accessor_derive::FieldAccessor;
use mongodb::{
  bson::{doc, from_bson, Bson, Document},
  options::{FindOneAndUpdateOptions, FindOneOptions},
};
use node_rs::{
  builder::GraphInternal,
  context::{Context, FlowContext},
  flow_message::PartialNodeOutput,
  graph::{Graph, InputRef, Node, OutputRef, Relation},
  main_error::MainResult,
  output_processor::{DefaultOutputProcessor, OutputProcessing},
  rabbitmq::FieldAccessor,
  start_process::start_process,
  App,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
  ops::{Mul, Sub},
  sync::Arc,
};
use tracing::{debug, info, span, trace, Instrument};
use uuid::Uuid;
use yansi::Paint;

// curl -X DELETE -u admin:admin 'http://localhost:15672/api/queues/%2F/flow_queue_fact/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_join/contents' 'http://localhost:15672/api/queues/%2F/flow_queue_map/contents'

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
enum Fact<T: Sub + Mul> {
  #[serde(rename = "n")]
  N(T, T),
  #[serde(rename = "fact")]
  Fact(T),
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
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

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct VecFact<T: Sub + Mul> {
  i: Vec<T>,
}

impl From<VecFact<u64>> for FactInput<u64> {
  fn from(v: VecFact<u64>) -> Self {
    FactInput {
      i: Fact::N(v.i[0], 1),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct JoinInput<T: Sub + Mul> {
  i: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, FieldAccessor)]
struct JoinResult {
  i: Bson,
}

struct MapOutputProcessing;

impl OutputProcessing for MapOutputProcessing {
  type INPUT = VecFact<u64>;
  type OUTPUT = FactInput<u64>;

  #[tracing::instrument(level = "debug", skip_all)]
  fn process(&self, out: Self::INPUT, context: Arc<Context>) -> Vec<(Self::OUTPUT, Arc<Context>)> {
    let map_id = &Uuid::new_v4().to_string();
    match context.get_label("maps".into()) {
      None => {
        context.insert_label("maps".into(), json!([map_id]));
      }
      Some(maps) => {
        if let Value::Array(mut maps) = maps {
          maps.insert(0, json!(map_id));
        }
      }
    };

    trace!(
      "{:?}:{:?}",
      Paint::red(&map_id),
      context.get_label("maps".into())
    );

    let size = out.i.len();

    out
      .i
      .into_iter()
      .enumerate()
      .map(|(i, o)| {
        let ctx: Context = (*context).clone();
        let meta = json! ({"index":i,"size":size});
        let l: String = "map-".to_string() + map_id;
        ctx.insert_label(l, meta);
        trace!(
          "ctx after op {:?} {:?} {:?}",
          i,
          Paint::cyan(&ctx),
          Paint::red(&context)
        );
        (FactInput { i: Fact::N(o, 1) }, Arc::new(ctx))
      })
      .collect()
  }
}

#[tokio::main]
async fn main() -> MainResult<()> {
  node_rs::tracing::init();

  let graph = Graph {
    id: String::from("fact"),
    nodes: vec![
      Node {
        id: "fact".to_string(),
        name: String::from("Factorial"),
        input: vec!["i".to_string()],
        output: vec!["o".to_string(), "r".to_string()],
      },
      Node {
        id: "map".to_string(),
        name: String::from("Map"),
        input: vec!["i".to_string()],
        output: vec!["i".to_string()],
      },
      Node {
        id: "join".to_string(),
        name: String::from("Join"),
        input: vec!["i".to_string()],
        output: vec!["i".to_string()],
      },
    ],
    edges: vec![
      Relation {
        from: OutputRef {
          node: "start".to_string(),
          output: "i".to_string(),
        },
        to: InputRef {
          node: "map".to_string(),
          input: "i".to_string(),
        },
      },
      Relation {
        from: OutputRef {
          node: "map".to_string(),
          output: "i".to_string(),
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
      Relation {
        from: OutputRef {
          node: "fact".to_string(),
          output: "r".to_string(),
        },
        to: InputRef {
          node: "join".to_string(),
          input: "i".to_string(),
        },
      },
    ],
  };

  let app = App::new(&graph.id).await;
  info!("{:?}", app.settings);
  let gi: GraphInternal = GraphInternal::new(graph.clone(), app.clone());

  #[tracing::instrument(level = "debug")]
  async fn map_action(
    data: VecFact<u64>,
    context: Arc<Context>,
  ) -> (Option<VecFact<u64>>, Arc<Context>) {
    debug!("map action {:?}", data);
    (Some(data), context)
  }

  gi.register_node_action("map", map_action, Some(Arc::new(MapOutputProcessing)))
    .await;

  #[tracing::instrument(level = "debug")]
  async fn fact_action(
    data: FactInput<u64>,
    context: Arc<Context>,
  ) -> (Option<FactResult<u64>>, Arc<Context>) {
    debug!("fact action {:?}", data);
    match data.i {
      Fact::N(1, r) => {
        info!("Fact: {:?}", r);
        (Some(FactResult::R(r)), context)
      }
      Fact::N(n, r) => (Some(FactResult::O(Fact::N(n - 1, r * n))), context),
      Fact::Fact(_) => (None, context),
    }
  }
  gi.register_node_action(
    "fact",
    fact_action,
    None::<Arc<DefaultOutputProcessor<FactResult<u64>, FactResult<u64>>>>,
  )
  .await;

  #[tracing::instrument(level = "debug")]
  async fn join_function(
    data: JoinInput<u64>,
    context: Arc<Context>,
  ) -> (Option<JoinResult>, Arc<Context>) {
    debug!("join action {:?}", data);
    let collection = context.app.as_ref().unwrap().runs_collection.clone();
    let process = collection
      .find_one(
        doc! {"processId":context.process_id.clone()},
        FindOneOptions::default(),
      )
      .instrument(span!(tracing::Level::DEBUG, "mongo.find_one"))
      .await
      .unwrap() // result
      .unwrap(); // Option
    if let Some(mapvec) = context.get_label("maps".into()) {
      let mapvec: Vec<String> = serde_json::from_value(mapvec).unwrap();
      let map_id = &mapvec[mapvec.len() - 1];
      let inputs = process
        .get_array("outputs")
        .unwrap()
        .iter()
        .filter(|&o| {
          let node_output: PartialNodeOutput = from_bson(o.clone()).unwrap();
          // TS:     if (o.output.node !== from.node || o.output.output !== from.output) return false;
          if node_output.output.node != context.relation.from.node
            || node_output.output.output != context.relation.from.output
          {
            return false;
          }

          if let Ok(context) = o.as_document().unwrap().get_document("context") {
            if let Ok(labels) = context.get_document("labels") {
              if let Ok(maps) = labels.get_array("maps") {
                // o.context?.labels?.maps
                return maps.iter().any(|map| {
                  // maps.contains(mapId)
                  if let Bson::String(string) = map {
                    string == map_id
                  } else {
                    false
                  }
                });
              }
            }
          }
          false
        })
        .collect::<Bson>();

      let map_label = "map-".to_string() + map_id;
      if let Some(labels) = context.get_label(map_label.clone()) {
        // labels[mapLabel].size
        let map_size = labels
          .as_object()
          .unwrap()
          .get("size")
          .unwrap()
          .as_u64()
          .unwrap();
        if map_size as usize == inputs.as_array().unwrap().len() {
          // Here we can process the join.
          let mut filter_query: Document = doc! {"processId":context.process_id.clone()};
          let join_key: String = "join-".to_string() + map_id;
          filter_query.insert(&join_key, doc! {"$exists":false});
          let mut update_query: Document = Document::new();
          update_query.insert(&join_key, 1); // seal the join in case of concurent join.
          collection
            .find_one_and_update(
              filter_query,
              doc! {"$set":  update_query},
              FindOneAndUpdateOptions::default(),
            )
            .instrument(span!(tracing::Level::DEBUG, "mongo.find_one_and_update"))
            .await
            .unwrap_or_default()
            .unwrap_or_default(); // TS:  if (result.value !== null)
          let mut im = inputs.as_array().unwrap().clone();
          let map_label_cloned = &map_label.clone();
          im.sort_by_cached_key(|i| -> i64 {
            i.as_document()
              .unwrap()
              .get_document("context")
              .unwrap()
              .get_document("labels")
              .unwrap()
              .get_document(map_label_cloned)
              .unwrap()
              .get("index")
              .unwrap()
              .as_i64()
              .unwrap()
          });
          let output_array = im
            .iter()
            .map(|i| i.as_document().unwrap().get("parameter").unwrap())
            .collect::<Bson>();
          context.remove_label(map_label);
          if let Some(mut mapvec) = context.get_label("maps".into()) {
            if let Value::Array(ref mut maps) = mapvec {
              maps.remove(0);
            }
            context.insert_label("maps".into(), mapvec);
            info!("Join: {}", output_array);
            return (Some(JoinResult { i: output_array }), context);
          }
        }
      }
    }
    (None, context)
  }
  gi.register_node_action(
    "join",
    join_function,
    None::<Arc<DefaultOutputProcessor<JoinResult, JoinResult>>>,
  )
  .await;

  let init = VecFact { i: vec![3, 10, 5] };
  start_process(app, gi, &init, &Some(FlowContext::default())).await;

  loop {
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
}

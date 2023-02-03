use crate::{
  graph::{Node, Relation},
  settings::Settings,
};
use log::debug;
use mongodb::{bson::{doc, Document},options::FindOneOptions, Collection};
use serde_json::Value;
use std::collections::HashMap;
use yansi::Paint;
lazy_static! {
  static ref SETTINGS: Settings = Settings::new().unwrap();
}

pub async fn fetch_parameter(
  process_id: &str,
  node: &Node,
  parameter: &mut Value,
  origin_map: &HashMap<String, Vec<Relation>>,
  events_collection: &Collection<Document>,
) {
  for i in &node.input {
    if !&parameter[i].is_null() {
      continue;
    }

    match origin_map.get(&(SETTINGS.rabbitmq.routing_key.clone() + "." + &node.id + "_" + i)) {
      None => {}
      Some(relation) => {
        let query_filter = doc! {"type": "updateOne",
          "collection": "runs",
          "filter": {
        "processId":process_id,
          },
          "update._push.outputs.output": {
        "node": &relation[0].from.node, // FIXME iterate over array
        "output": &relation[0].from.output,
          },};
        // TODO: filter more with inputMatching

        let event_document = events_collection
          .find_one(query_filter, FindOneOptions::default())
          .await
          .unwrap() // OK
          .unwrap(); // Option
        let parameter_in_db = event_document
          .get_document("update")
          .unwrap()
          .get_document("_push")
          .unwrap()
          .get_document("outputs")
          .unwrap()
          .get("parameter")
          .unwrap();
        debug!("{:?} = {:?}", i, Paint::green(parameter_in_db));
        let v = serde_json::to_value(parameter_in_db).unwrap();
        parameter[i] = v;
      }
    }
  }
}

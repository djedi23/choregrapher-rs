use crate::{
  builder::GraphInternal,
  context::FlowContext,
  db,
  flow_message::FlowMessage,
  graph::Node,
  rabbitmq::{send_message, FieldAccessor},
  routing::routing,
  settings::Settings,
};
use bson::{doc, to_bson, Bson};
use chrono::Utc;
use lapin::Channel;
use log::trace;
use mongodb::options::{UpdateModifications, UpdateOptions};
use serde::Serialize;
use std::fmt::Debug;
use uuid::Uuid;

/// Start a new choragrapher process.
/// returns the process id.
///  partof: #SPC-run.startProcess
pub async fn start_process<T>(
  chan: &Channel,
  graph: GraphInternal<'_>,
  init: &T,
  context: &Option<FlowContext>,
) -> String
//lapin::Result<Confirmation>
where
  T: Debug + Clone + FieldAccessor + Serialize,
{
  let process_id = &Uuid::new_v4().to_string();
  let settings = Settings::new().unwrap();

  for output_field in init.clone().field() {
    trace!("start process: field {:?}", &output_field);
    if let Bson::Document(init) = to_bson(&init).unwrap() {
      let mut options = UpdateOptions::default();
      options.upsert = Some(true);
      let now = Utc::now();
      let parameter = init.get(output_field.clone().as_str()).unwrap();

      // partof: #SPC-run.initProcessInDB
      let collection = db::get_collection(&settings).await.unwrap();
      collection
        .update_one(
          doc! {"processId":process_id.clone()},
          UpdateModifications::Document(doc! {
          "$set": {
                          "processId": process_id.clone(),
                          "graphId": graph.id.clone(),
                          "timestamp": now,
          },
                      "$push":{
                          "outputs":
                          {
                              "processId": process_id.clone(),
                              "timestamp": now,
                  "output": {
                "node": settings.starting_node_id.clone(),
                "output": output_field.clone(),
                  },
                  "parameter": parameter,
                          }
                      }
                  }),
          Some(options),
        )
        .await
        .unwrap();

      let collection = db::get_events_collection(&settings).await.unwrap();
      collection
        .insert_one(
          doc! {
              "type": "updateOne",
              "collection": settings.database.collection.clone(),
              "filter": {
                  "processId": process_id.clone()
              },
              "update": {
                  "_push": {
                      "outputs": {
                          "processId": process_id.clone(),
                          "output": {
                              "node": settings.starting_node_id.clone(),
                              "output": output_field.clone()
                          },
                          "parameter": parameter,
                          "timestamp": now,
                      }
                  },
                  "_set": {
                      "processId": process_id.clone(),
                      "graphId": graph.id.clone()
                  }
              },
              "options": {
                  "upsert": true
              },
              "timestamp": Utc::now(),
          },
          None,
        )
        .await
        .unwrap();
    }
  }
  for route in routing(
    &graph.destination_map,
    &Node::id(settings.starting_node_id.clone()),
    init,
  ) {
    trace!("start process: route {:?}", &route);
    let payload = FlowMessage {
      process_id: process_id.clone(),
      parameter: &route.payload,
      context: context.clone(),
    };

    send_message(
      chan,
      &graph.id,
      &route.topic,
      serde_json::to_vec::<FlowMessage<&serde_json::Value>>(&payload).unwrap(),
    )
    .await
    .unwrap();
  }
  process_id.clone()
}

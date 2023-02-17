use crate::{
  builder::GraphInternal,
  context::FlowContext,
  flow_message::FlowMessage,
  graph::Node,
  rabbitmq::{send_message, FieldAccessor},
  routing::routing,
  settings::Settings,
  App,
};
use chrono::Utc;
use mongodb::{
  bson::{doc, to_bson, Bson},
  options::{UpdateModifications, UpdateOptions},
};
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};
use tracing::{instrument, span, trace, Instrument};
use uuid::Uuid;

/// Start a new choragrapher process.
/// returns the process id.
///  partof: #SPC-run.startProcess
#[instrument(skip(app, graph))]
pub async fn start_process<T>(
  app: Arc<App>,
  graph: GraphInternal,
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
      let collection = &app.runs_collection;
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
        .instrument(span!(tracing::Level::DEBUG, "mongo.update_one"))
        .await
        .unwrap();

      let collection = &app.events_collection;
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
        .instrument(span!(tracing::Level::DEBUG, "mongo.insert_one"))
        .await
        .unwrap();
    }
  }
  for route in routing(
    &graph.destination_map,
    &Node::id(settings.starting_node_id.clone()),
    init,
    app.settings.clone(),
  ) {
    trace!("start process: route {:?}", &route);
    let payload = FlowMessage {
      process_id: process_id.clone(),
      parameter: &route.payload,
      context: context.clone(),
    };

    send_message(
      app.clone(),
      &graph.id,
      &route.topic,
      serde_json::to_vec::<FlowMessage<&serde_json::Value>>(&payload).unwrap(),
    )
    .await
    .unwrap();
  }
  process_id.clone()
}

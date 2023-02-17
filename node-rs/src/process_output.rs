use crate::{
  context::Context,
  flow_message::FlowMessage,
  graph::{InputRef, Node},
  output_processor::OutputProcessing,
  rabbitmq::{send_message, FieldAccessor},
  routing::routing,
  App,
};
use chrono::Utc;
use mongodb::{
  bson::{doc, to_bson, Bson},
  options::UpdateModifications,
};
use serde::Serialize;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tracing::{instrument, span, trace, Instrument};
use yansi::Paint;

#[allow(clippy::borrowed_box, clippy::too_many_arguments)]
#[instrument(level = "debug", skip(output_processor, app))]
pub async fn process_output<'a, R, O>(
  result: Option<R>,
  context: Arc<Context>,
  graph_id: &str,
  process_id: &str,
  node: &Node,
  destination_map: &'a HashMap<String, Vec<InputRef>>,
  output_processor: &Arc<impl OutputProcessing<INPUT = R, OUTPUT = O> + ?Sized + 'static>,
  app: Arc<App>,
) where
  R: Debug + Serialize + FieldAccessor + Clone,
  O: Debug + Serialize + FieldAccessor + Clone,
{
  if let Some(result) = result {
    let collection = app.runs_collection.clone();

    // partof: #SPC-processing.node_outputprocessing
    for (result, context) in output_processor.process(result, context) {
      trace!(
        "after output processor {:?} {:?}",
        Paint::green(&result).bold(),
        context
      );
      if let Bson::Document(result_bson) = to_bson(&result).unwrap() {
        let bson_context = to_bson(&context.context).unwrap();
        for field in result.clone().field() {
          let now = Utc::now();
          // partof: #SPC-processing.saveOutput
          let parameter = result_bson.get(field.as_str()).unwrap();
          collection
            .update_one(
              doc! {"processId":process_id},
              UpdateModifications::Document(doc! {
                                      "$push":{
                                          "outputs":
                                          {
                                              "processId": process_id,
                                              "timestamp": now,
                "output": {
              "node": &node.id,
              "output": &field,
                },
                "parameter": parameter,
                "context": &bson_context
                                          }
                                      }
                }),
              None,
            )
            .instrument(span!(tracing::Level::DEBUG, "mongo.update_one"))
            .await
            .unwrap();

          app
            .events_collection
            .insert_one(
              doc! {
                  "type": "updateOne",
                  "collection": &app.settings.database.collection,
                  "filter": {
                "processId": process_id
                  },
                  "update": {
                "_push": {
                    "outputs": {
                  "processId": process_id,
                  "output": {
                      "node": &node.id,
                      "output": &field,
                  },
                  "parameter": parameter,
                  "timestamp": now,
                  "context": &bson_context
                    }
                }
                  },
                  "options": null,
                  "timestamp": Utc::now(),
              },
              None,
            )
            .instrument(span!(tracing::Level::DEBUG, "mongo.insert_one"))
            .await
            .unwrap();
        }
      }

      // partof: #SPC-processing.callRoutingOutput
      for route in routing(destination_map, node, &result, app.settings.clone()) {
        let payload = FlowMessage {
          process_id: process_id.to_string(),
          parameter: &route.payload,
          context: Some(Arc::clone(&context).context.clone()),
        };

        // partof: #SPC-processing.sendOutput
        send_message(
          app.clone(),
          graph_id,
          &route.topic,
          serde_json::to_vec::<FlowMessage<&serde_json::Value>>(&payload).unwrap(),
        )
        .await
        .unwrap();
      }
    }
  }
}

use crate::{
  context::Context,
  db,
  fetchparameter::fetch_parameter,
  flow_message::PartialFlowMessage,
  graph::{InputRef, Node, Relation},
  output_processor::OutputProcessing,
  process_output::process_output,
  settings::Settings,
};
use lapin::{
  message::DeliveryResult,
  options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
  },
  publisher_confirm::Confirmation,
  types::FieldTable,
  BasicProperties, Channel, Connection, ConnectionProperties, Error, ExchangeKind,
};
use mongodb::bson::doc;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fmt::Debug, future::Future, sync::Arc};
use tracing::{debug, info, instrument, span, trace, Instrument};

lazy_static! {
  static ref SETTINGS: Settings = Settings::new().unwrap();
}

pub trait FieldAccessor {
  fn field_list(self) -> Vec<String>;
  /// Return a string representation of the field
  ///
  fn field(self) -> Vec<String>;
}

pub async fn create_rabbit_mq(exchange_suffix: &str) -> Result<(Channel, Channel), Error> {
  info!("Rabbitmq init");
  let rabbitmq_url = std::env::var("AMQP_ADDR").unwrap_or_else(|_| SETTINGS.rabbitmq.url.clone());
  let options = ConnectionProperties::default()
    // Use tokio executor and reactor.
    // At the moment the reactor is only available for unix.
    .with_executor(tokio_executor_trait::Tokio::current())
    .with_reactor(tokio_reactor_trait::Tokio);
  let connection = Connection::connect(&rabbitmq_url, options).await?;

  let channel = connection.create_channel().await?;
  let channel_out = connection.create_channel().await?;
  let exchange_name = (SETTINGS.rabbitmq.exchange_name.to_owned()) + "_" + exchange_suffix;

  debug!("rabbitmq.exchange_name = {}", &exchange_name);

  let exchange_declare_options = ExchangeDeclareOptions {
    durable: true,
    ..Default::default()
  };
  // ExchangeDeclareOptions::default();
  //  exchange_declare_options.durable = true;
  // partof: #SPC-rabbitmq.exchange
  channel
    .exchange_declare(
      &exchange_name,
      ExchangeKind::Topic,
      exchange_declare_options,
      FieldTable::default(),
    )
    .await?;

  // partof: #SPC-rabbitmq.prefetch
  channel
    .basic_qos(SETTINGS.rabbitmq.prefetch_count, BasicQosOptions::default())
    .await?;

  //    info!("Declared queue {:?}", queue);

  Ok((channel, channel_out))
}

#[rustfmt::skip::macros(doc)]
#[allow(clippy::too_many_arguments)]
pub async fn create_consumer<'de, Action, T, R, O, F>(
  channel: &Channel,
  channel_out: &Channel,
  destination_map: &HashMap<String, Vec<InputRef>>,
  origin_map: &HashMap<String, Vec<Relation>>,
  graph_id: &str,
  node: &Node,
  action: Action,
  output_processor: Arc<
    impl OutputProcessing<INPUT = R, OUTPUT = O> + ?Sized + Sync + Send + 'static,
  >,
) where
  T: Debug + DeserializeOwned + Send,
  R: Debug + Serialize + FieldAccessor + Clone + Send + Sync,
  O: Debug + Serialize + FieldAccessor + Clone + Send + Sync,
  F: Future<Output = (Option<R>, Arc<Context>)> + Send,
  Action: Fn(T, Arc<Context>) -> F + 'static + Sync + Send + Clone + Copy,
{
  let queue_declare_options = QueueDeclareOptions {
    durable: true,
    ..Default::default()
  };
  let queue_name = SETTINGS.rabbitmq.queue_name.clone() + "_" + &node.id;
  let exchange_name = (SETTINGS.rabbitmq.exchange_name.to_owned()) + "_" + graph_id;
  // partof: #SPC-rabbitmq #SPC-rabbitmq.nodeHaveQueue
  channel
    .queue_declare(&queue_name, queue_declare_options, FieldTable::default())
    .await
    .unwrap();

  for input in &node.input {
    // partof: #SPC-rabbitmq.inputBindTopic
    channel
      .queue_bind(
        &queue_name,
        &exchange_name,
        &(SETTINGS.rabbitmq.routing_key.clone() + "." + &node.id + "_" + input),
        QueueBindOptions::default(),
        FieldTable::default(),
      )
      .await
      .unwrap();
  }

  let consumer = channel
    .clone()
    .basic_consume(
      &queue_name,
      &node.name, // FIXME: ensure the tag is unique
      BasicConsumeOptions::default(),
      FieldTable::default(),
    )
    .await
    .unwrap();

  // let channel2 = channel.clone();
  let channel_out = channel_out.clone();
  let node = node.clone();
  let destination_map = destination_map.clone();
  let origin_map = origin_map.clone();
  let graph_id = graph_id.to_owned();
  let handle = tokio::runtime::Handle::current();

  info!("will consume");
  consumer.set_delegate(move |delivery: DeliveryResult| {
    let channel_out2 = channel_out.clone();
    let origin_map = origin_map.clone();
    let node = node.clone();
    let queue_name = queue_name.clone();
    let graph_id = graph_id.to_owned();
    let destination_map = destination_map.clone();
    let output_processor = output_processor.clone();
    let handle = handle.clone();

    async move {
      handle.spawn(
        async move {
          let events_collection = db::get_events_collection(&SETTINGS).await.unwrap();

          //      while let Some(delivery) = consumer.next().await {
          // partof: #SPC-processing
          let delivery = match delivery {
            // Carries the delivery alongside its channel
            Ok(Some(delivery)) => delivery,
            // The consumer got canceled
            Ok(None) => return,
            // Carries the error and is always followed by Ok(None)
            Err(error) => {
              dbg!("Failed to consume queue message {}", error);
              return;
            }
          }; //           let Some( delivery) = delivery.expect("error caught in consumer")
             // else { panic!("Empty rabbitmq delivery") }; // FIXME: do better than panic.
          trace!(
            "Consume [ {} ] {:?} {:?}",
            queue_name,
            delivery,
            String::from_utf8(delivery.data.clone())
          );

          let mut relation = None;
          for relationf in origin_map.get(&delivery.routing_key.to_string()).unwrap() {
            if relationf.to.node == node.id {
              relation = Some(relationf);
              break;
            }
          }
          let relation: Relation = relation.unwrap().clone();

          let mut data: PartialFlowMessage = serde_json::from_slice(&delivery.data).unwrap();
          debug!("Data {:?}", data);

          // partof: #SPC-processing.missingInput
          fetch_parameter(
            &data.process_id,
            &node,
            &mut data.parameter,
            &origin_map,
            &events_collection,
          )
          .await;

          // partof: #SPC-processing.actionContext
          let context = Context {
            process_id: data.process_id.clone(),
            relation,
            context: data.context.clone().unwrap_or_default(),
          };

          // partof: #SPC-processing.callAction
          let (result, context): (Option<R>, Arc<Context>) = action(
            serde_json::from_value::<T>(data.parameter).unwrap(),
            Arc::new(context),
          )
          .await;

          process_output(
            result,
            context,
            &graph_id,
            &data.process_id,
            &node,
            &destination_map,
            &channel_out2,
            &output_processor,
            &events_collection,
          )
          .await;
          delivery.ack(BasicAckOptions::default()).await.expect("ack");
        }
        .instrument(span!(tracing::Level::INFO, "msg handler")),
      );
    }
  });
}

#[instrument(level = "debug")]
pub async fn send_message(
  channel: &Channel,
  graph_id: &str,
  topic: &str,
  payload: Vec<u8>,
) -> lapin::Result<Confirmation> {
  let publish_properties = BasicProperties::default();
  let publish_properties = publish_properties.with_delivery_mode(2);
  let exchange_name = (SETTINGS.rabbitmq.exchange_name.to_owned()) + "_" + graph_id;
  trace!(
    "Send Message [ {} : {} ] {:?} {:?}",
    &exchange_name,
    topic,
    String::from_utf8(payload.clone()),
    channel
  );

  channel
    .basic_publish(
      &exchange_name,
      topic,
      BasicPublishOptions::default(),
      &payload,
      publish_properties,
    )
    .await?
    .await
}

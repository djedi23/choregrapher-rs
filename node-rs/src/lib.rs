//! # Choragrapher node
//!
//! `node-rs`

use crate::db::get_graph_collection;
use lapin::Channel;
use mongodb::{bson::Document, Collection};
use settings::Settings;
use std::sync::Arc;

pub mod builder;
pub mod context;
pub mod db;
mod fetchparameter;
pub mod flow_message;
pub mod graph;
pub mod macros;
pub mod main_error;
pub mod output_processor;
mod process_output;
pub mod rabbitmq;
mod routing;
pub mod settings;
pub mod start_process;
pub mod tracing;

#[derive(Debug)]
pub struct App {
  pub settings: Arc<Settings>,
  pub channel: Arc<Channel>,
  pub channel_out: Arc<Channel>,
  pub runs_collection: Arc<Collection<Document>>,
  pub events_collection: Arc<Collection<Document>>,
  pub graph_collection: Arc<Collection<Document>>,
}

impl App {
  pub async fn new(exchange_suffix: &str) -> Arc<Self> {
    let settings = Arc::new(Settings::new().unwrap());
    App::new_with_setting(exchange_suffix, settings).await
  }

  pub async fn new_with_setting(exchange_suffix: &str, settings: Arc<Settings>) -> Arc<Self> {
    use db::{get_collection, get_events_collection};
    use rabbitmq::create_rabbit_mq;
    let (channel, channel_out) = create_rabbit_mq(exchange_suffix, settings.clone())
      .await
      .expect("Can't connect to RabbmitMQ"); // FIXME: remove expect

    Arc::new(App {
      runs_collection: get_collection(&settings).await.unwrap().into(),
      events_collection: get_events_collection(&settings).await.unwrap().into(),
      graph_collection: get_graph_collection(&settings).await.unwrap().into(),
      settings,
      channel,
      channel_out,
    })
  }
}

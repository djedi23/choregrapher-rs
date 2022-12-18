//! # Setting of the application
//! Manage the settings of the application
//!
//! You can modify the config with a file `config/local.toml`.  The
//! settings can be overrided by the environment var with prefix
//! `NODE_` and separated with `__` (ex: `NODE_RABBITMQ__PREFETCH_COUNT=2` )
//!
//! # Examples
//! ```
//! use node_rs::settings::Settings;
//! let settings = Settings::new().unwrap();
//! println!("{:?}",settings);
//! ```

use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// Database configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
  /// mongo connection url
  pub url: String,
  /// Database to use
  pub database: String,
  /// Collection where the processes are saved.
  pub collection: String,
  /// Collection where the events are saved.
  pub events_collection: String,
  /// Collection where the graphes are saved.
  pub graphes_collection: String,
  /// bucket where the files are saved.
  pub files_bucket: String,
}

impl Default for Database {
  fn default() -> Self {
    Database {
      url: "mongodb://localhost:27017".into(),
      database: "test".into(),
      collection: "runs".into(),
      events_collection: "events".into(),
      graphes_collection: "graph".into(),
      files_bucket: "choregrapher".into(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RabbitMQ {
  pub url: String,
  pub exchange_name: String,
  pub prefetch_count: u16,
  pub queue_name: String,
  pub routing_key: String,
}
impl Default for RabbitMQ {
  fn default() -> Self {
    RabbitMQ {
      url: "amqp://admin:admin@127.0.0.1:5672/%2f".into(),
      exchange_name: "flow".into(),
      prefetch_count: 8,
      queue_name: "flow_queue".into(),
      routing_key: "flow".into(),
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
  pub debug: bool,
  pub starting_node_id: String,
  pub database: Database,
  pub rabbitmq: RabbitMQ,
}

impl Default for Settings {
  fn default() -> Self {
    Settings {
      debug: false,
      starting_node_id: "start".into(),
      database: Database::default(),
      rabbitmq: RabbitMQ::default(),
    }
  }
}

impl Settings {
  pub fn load() -> Result<Config, ConfigError> {
    let mut s = Config::try_from(&Settings::default())?;
    s.merge(File::with_name("~/.config/dancer/config/default").required(false))?;
    // Start off by merging in the "default" configuration file
    s.merge(File::with_name("config/default").required(false))?;

    // Add in the current environment file
    // Default to 'development' env
    // Note that this file is _optional_
    let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

    // Add in a local configuration file
    // This file shouldn't be checked in to git
    s.merge(File::with_name("config/local").required(false))?;

    // Add in settings from the environment (with a prefix of APP)
    // Eg.. `NODE_DEBUG=1 ./target/app` would set the `debug` key
    s.merge(Environment::with_prefix("NODE").separator("__"))?;

    // You may also programmatically change settings
    //        s.set("database.url", "postgres://")?;

    // Now that we're done, let's access our configuration
    // println!("debug: {:?}", s.get_bool("debug"));
    // println!("database: {:?}", s.get::<String>("database.url"));

    Ok(s)
  }
  pub fn new() -> Result<Self, ConfigError> {
    match Settings::load() {
      Err(e) => Err(e),
      Ok(s) => {
        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
      }
    }
  }
}

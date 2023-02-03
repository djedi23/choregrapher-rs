use crate::settings::Settings;
use anyhow::{Context, Result};
use mongodb::{bson::Document, options::ClientOptions, Client, Collection, Database};

pub async fn get_database(settings: &Settings) -> Result<Database> {
  let mongo_client_options = ClientOptions::parse(&settings.database.url)
    .await
    .context(format!("Can't parse mongo url: {}", &settings.database.url))?;
  let mongo_client =
    Client::with_options(mongo_client_options).context("Can't create mongo client")?;
  Ok(mongo_client.database(&settings.database.database))
}

pub async fn get_collection(settings: &Settings) -> Result<Collection<Document>> {
  let db = get_database(settings)
    .await
    .context("Can't get a database")?;
  Ok(db.collection(&settings.database.collection))
}

pub async fn get_events_collection(settings: &Settings) -> Result<Collection<Document>> {
  let db = get_database(settings)
    .await
    .context("Can't get a database")?;
  Ok(db.collection(&settings.database.events_collection))
}

pub async fn get_graph_collection(settings: &Settings) -> Result<Collection<Document>> {
  let db = get_database(settings)
    .await
    .context("Can't get a database")?;
  Ok(db.collection(&settings.database.graphes_collection))
}

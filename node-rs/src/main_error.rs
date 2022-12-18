pub type MainResult<T> = std::result::Result<T, MainError>;

#[derive(Debug)]
pub enum MainError {
  LapinError(lapin::Error),
  MongoError(mongodb::error::Error),
}

impl From<mongodb::error::Error> for MainError {
  fn from(err: mongodb::error::Error) -> MainError {
    MainError::MongoError(err)
  }
}

impl From<lapin::Error> for MainError {
  fn from(err: lapin::Error) -> MainError {
    MainError::LapinError(err)
  }
}

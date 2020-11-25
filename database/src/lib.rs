use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
  #[error("Value {0} ({1}) not found")]
  ValueNotFound(String, &'static str),
}

mod language;
pub use language::*;
mod user;
pub use user::*;

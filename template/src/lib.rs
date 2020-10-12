use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Word {
  pub subwords: Vec<String>,
  pub mutation: Option<HashMap<String, String>>,
  pub tags: Vec<String>,
  pub value: Option<String>,
}

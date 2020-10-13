use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Word {
  pub subwords: Vec<String>,
  pub mutation: Option<HashMap<String, String>>,
  pub tags: Vec<String>,
  pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Params {
  pub com: String,
  pub args: HashMap<String, Vec<String>>,
}

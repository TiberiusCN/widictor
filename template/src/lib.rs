use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateText {
  pub subwords: Vec<String>,
  pub mutation: Option<HashMap<String, String>>,
  pub tags: Vec<String>,

  pub lemma: Option<String>,
  pub conjugation: Option<String>,
  pub declension: Option<String>,
  pub pronunciation: Option<String>,
  pub meanings: Option<String>,
  pub examples: Option<String>,
  pub notes: Option<String>,
}

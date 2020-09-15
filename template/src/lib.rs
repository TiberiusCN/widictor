use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Subword {
  pub language: String,
  pub word: String,
}

#[derive(Serialize, Deserialize)]
pub struct TemplateText {
  pub subwords: Vec<Subword>,
  pub conjugation: Option<String>,
  pub text: String,
}

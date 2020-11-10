use serde_derive::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[repr(u32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SectionSpecies {
  Unknown,
  Word,
  Mutation,
  Provided,
  Etymology,
  Pronunciation,
  UsageNotes,
}


#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Word {
  pub derived: HashSet<String>,
  pub produced: HashSet<String>,
  pub mutation: Option<HashMap<String, String>>,
  pub tags: HashSet<String>,
  pub value: Option<String>,
  pub properties: HashMap<String, String>,
}

impl std::ops::AddAssign for Word {
  fn add_assign(&mut self, other: Self) {
    for derived in other.derived {
      self.derived.insert(derived);
    }
    for produced in other.produced {
      self.produced.insert(produced);
    }
    if let Some(value) = other.value {
      self.append_value("", &value, "");
    }
    if let Some(mutations) = other.mutation {
      self.mutation = Some(mutations); // rewrite is ok
    }
    for tag in other.tags {
      self.tags.insert(tag);
    }
    for property in other.properties {
      self.properties.insert(property.0, property.1);
    }
  }
}

impl Word {
  /// This function will ignore void value if self.value isn't exists. It allows you to ignore leading spaces and empty strings.
  pub fn append_value(&mut self, prefix: &str, value: &str, suffix: &str) {
    if value.trim().is_empty() && self.value.is_none() { return; }
    self.add_value(&format!("{}{}{}", prefix, value, suffix));
  }

  pub fn add_value(&mut self, value: &str) {
    self.value = Some(self.value.take().unwrap_or_default() + value);
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Params {
  pub section: SectionSpecies,
  pub com: String,
  pub args: HashMap<String, String>,
}

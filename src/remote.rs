use serde::*;
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
  #[error(transparent)]
  Serde(#[from] serde_json::Error),
  #[error("something missing in wikitext")]
  LackOfData,
}
  
#[derive(Deserialize)]
struct ApiAnswer {
  query: ApiQuery,
}

#[derive(Deserialize)]
struct ApiQuery {
  pages: HashMap<String, ApiPage>,
}

#[derive(Deserialize)]
struct ApiPage {
  //pageid: u32,
  //ns: u32,
  //title: String,
  revisions: Vec<ApiRevision>,
}

#[derive(Deserialize)]
struct ApiRevision {
  //contentformat: String,
  //contentmodel: String,
  #[serde(rename = "*")]
  data: String,
}

pub fn get(page: &str) -> Result<String, Error> {
  let resp = reqwest::blocking::get(&format!("https://en.wiktionary.org/w/api.php?action=query&prop=revisions&rvprop=content&format=json&titles={}", page))?;
  let resp: ApiAnswer = serde_json::from_reader(resp.bytes().unwrap().as_ref())?;
  Ok(resp.query.pages.iter().last().ok_or_else(|| Error::LackOfData)?.1.revisions[0].data.clone())
}

pub fn translate(s: &str, into: &str) -> String {
  let s = percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string();
  let resp = reqwest::blocking::get(&format!("http://translate.googleapis.com/translate_a/single?client=gtx&sl=EN&tl={}&dt=t&q={}", into, s)).unwrap();
  let resp = std::string::String::from_utf8(resp.bytes().unwrap().as_ref().to_owned()).unwrap();
  let mut out = String::new();
  let mut q = false;
  let mut s = false;
  for c in resp.chars() {
    match c {
      '"' => if !s {
        q = !q;
        if !q { break; }
      },
      '\\' => s = !s,
      x => if q { out.push(x); },
    }
  }
  out
}

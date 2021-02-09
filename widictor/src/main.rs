use std::collections::HashMap;
/*
use nom::*;
use nom::error::*;
use template::{Word as Lemma, Params, SectionSpecies};
use std::io::Read;
use database::*;
*/

mod section;
mod word_section;
mod wiki_error;

lazy_static::lazy_static! {
  static ref TEMPLATES: HashMap<String, std::path::PathBuf> = {
    let config_path = directories::ProjectDirs::from("com", "apqm", "widictor").unwrap().config_dir().join("templates.conf");
    let x_dir = directories::BaseDirs::new().unwrap().executable_dir().unwrap().to_owned();
    let f = std::fs::read_to_string(config_path).unwrap();
    let mut hash = HashMap::new();
    for p in f.lines() {
      let mut  p = p.split('~');
      if let (Some(template), Some(executable)) = (p.next(), p.next()) {
        hash.insert(template.to_owned(), x_dir.join(executable));
      }
    }
    hash
  };
}

fn main() {
  let arg = std::env::args().nth(1).unwrap();
  let languages = vec![
    "Latin".to_owned(),
    "French".to_owned(),
    "Italian".to_owned(),
    "English".to_owned(),
    "German".to_owned(),
  ];
  scan(&arg, &languages);
}

fn scan(page: &str, languages: &[String]) {
  /*
  let mut wiki = Wiki::new(page, languages);
  let bases = Bases::new().unwrap();
  loop {
    match wiki.parse(&bases, 1) {
      Ok(false) => {
        break;
      },
      Ok(true) => {},
      Err(e) => {
        eprintln!("{}", e);
      }
    }
  }
  */
}

// {} — hide from translation
// [] — hide in tests
// _x_ — this word

/* ToDo:
  insert form even if no value
  template in template
*/

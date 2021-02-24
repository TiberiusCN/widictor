use std::{collections::{HashMap, HashSet}, rc::Rc};
use language::Language;
use text::Text;
use text::Piece;
use word_section::WordSection;
//use text::Text;
/*
use nom::*;
use nom::error::*;
use template::{Word as Lemma, Params, SectionSpecies};
use std::io::Read;
use database::*;
*/

pub(crate) mod substr;
mod section;
mod word_section;
mod wiki_error;
mod language;
mod text;

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

enum AnyParse<'a> {
  Language(Language<(), WordSection<()>>),
  Section(WordSection<()>),
  Content(&'a str),
}
impl<'a> AnyParse<'a> {
  fn parse(input: &'a str) -> Self {
    if let Ok(r) = Language::parse(input).map(|v| v.1).map(Self::Language) { r }
    else if let Ok(r) = WordSection::parse(input).map(|v| v.1).map(Self::Section) { r }
    else { Self::Content(input) }
  }
}

fn parse_page(page: &str, language: &str, subwords: &mut HashSet<String>) -> Result<Vec<Language<String, Rc<WordSection<String>>>>, ()> {
  let mut iter = page.lines();
  let mut lang = {
    let iter = &mut iter;
    (move || {
      while let Some(line) = iter.next() {
        match Language::parse(line) {
          Ok(lang) if lang.1.name == language => return Ok(lang.1),
          Ok(_) => {},
          Err(nom::Err::Error(e)) if !e.filtered() => return Err(()),
          Err(_) => {},
        }
      }
      Err(())
    })()?
  }.convert(|_| unreachable!());
  for line in iter {
    let any = AnyParse::parse(line);
    match any {
      AnyParse::Language(_) => break,
      AnyParse::Section(section) => {
        lang.sections.push(section.convert(|_| unreachable!()));
      }
      AnyParse::Content(content) => {
        lang.section().children.push(content.trim());
      }
    }
  }
  let lang = lang.fold_convert(|mut acc: Vec<String>, line: &str| -> Vec<String> {
    if line.is_empty() {
      acc.push(String::new());
    } else {
      if line.chars().next().map(|prefix| matches!(prefix, '#' | '*')).unwrap_or_default() {
        acc.push(line.to_owned());
      } else {
        if let Some(last) = acc.last_mut() {
          *last += " ";
          *last += line;
        } else {
          acc.push(line.to_owned());
        }
      }
    }
    acc
  });
  let lang = lang.try_convert(|src: String| -> Result<Text, ()> {
    Text::parse(&src, subwords).map_err(|e| {
      eprintln!("\x1b[31mError\x1b[0m: {:?} during parsing «{}»", e, src);
      ()
    }).map(|v| v.1)
  })?;
  let lang = lang.convert(|text: Text| -> String {
    let (mut line, pieces) = match text {
      Text::Text(pieces) => ("".to_owned(), pieces),
      Text::List(level, pieces) => {
        let mut prefix = String::new();
        for _ in 0..level {
          prefix += "*";
        }
        prefix += " ";
        (prefix, pieces)
      },
    };
    for (id, piece) in pieces.into_iter().enumerate() {
      let value = match piece {
        Piece::Raw(s) => s,
        Piece::Template(params) => format!("{{COM:{}}}", params.com),
      };
      line += format!("«{}:{}»", id, value).as_str();
    }
    line
  });
  Ok(lang.subdivide())
}

fn main() {
  let arg = std::env::args().nth(1).unwrap();
  scan(&arg);
}

fn scan(word: &str) {
  let page = remote::get(word).unwrap();
  let mut subwords = HashSet::new();
  let words = parse_page(&page, "French", &mut subwords).unwrap();
  for (id, page) in words.into_iter().enumerate() {
    println!("{} — {}:", word, id);
    for section in page.sections.iter().rev() {
      println!("  {:?}:", section.name);
      for child in &section.children {
        println!("    {}", child);
      }
    }
  }
}

// {} — hide from translation
// [] — hide in tests
// _x_ — this word

/* ToDo:
  insert form even if no value
  template in template
*/

use std::{collections::{HashMap, HashSet}, rc::Rc};
use language::Language;
use scribunto::{LuaInstance, LuaTable};
use text::Text;
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
  let lang = lang.try_convert(|src: String| -> Result<Vec<Text>, ()> {
    let mut src = src.as_str();
    let mut out = Vec::new();
    while !src.is_empty() {
      let (s, text) = Text::parse(&src, subwords).map_err(|e| {
        eprintln!("\x1b[31mError\x1b[0m: {:?} during parsing «{}»", e, src);
        ()
      })?;
      src = s;
      out.push(text);
    }
    Ok(out)
  })?;
  let lang = lang.convert(|text: Vec<Text>| -> String {
    let mut out = String::new();
    for text in text {
      if !out.is_empty() { out += " "; }
      match text {
        Text::Raw(raw) => out += raw.as_str(),
        Text::Tab(tab) => {
          for _ in (0..).take(tab as usize) {
            out += "»";
          }
        }
        Text::Template(template) => {
          out += format!("{{COM}}").as_str();
        }
      }
    }
    out
  });
  Ok(lang.subdivide())
}

fn main() {
  let mut machine = LuaInstance::new(
    "/usr/share/webapps/mediawiki/extensions/Scribunto/includes/engines/LuaStandalone/mw_main.lua",
    "/usr/share/webapps/mediawiki/extensions/Scribunto/includes",
    0,
    4,
    vec![
      "/usr/share/webapps/mediawiki/extensions/Scribunto/includes/engines/LuaCommon/lualib".to_owned(),
    ]
  ).unwrap();
  println!("{:#?}", machine.register_library("mw_interface", LuaTable::default()).unwrap());
  println!("{:#?}", machine.get_status().unwrap());
  println!("{:#?}", machine.load_file("mwInit_lua", "mwInit.lua").unwrap());
  println!("{:#?}", machine.get_status().unwrap());
  println!("{:#?}", machine.call(1, LuaTable::default()).unwrap());
  let mut table = LuaTable::<scribunto::LuaString>::default();
  table.insert_string("loadPackage", "mw_interface-loadPackage-2");
  table.insert_string("loadPHPLibrary", "mw_interface-loadPHPLibrary-2");
  table.insert_string("frameExists", "mw_interface-frameExists-2");
  table.insert_string("newChildFrame", "mw_interface-newChildFrame-2");
  table.insert_string("getExpandedArgument", "mw_interface-getExpandedArgument-2");
  table.insert_string("getAllExpandedArguments", "mw_interface-getAllExpandedArguments-2");
  table.insert_string("expandTemplate", "mw_interface-expandTemplate-2");
  table.insert_string("callParserFunction", "mw_interface-callParserFunction-2");
  table.insert_string("preprocess", "mw_interface-preprocess-2");
  table.insert_string("incrementExpensiveFunctionCount", "mw_interface-incrementExpensiveFunctionCount-2");
  table.insert_string("isSubsting", "mw_interface-isSubsting-2");
  table.insert_string("getFrameTitle", "mw_interface-getFrameTitle-2");
  table.insert_string("setTTL", "mw_interface-setTTL-2");
  table.insert_string("addWarning", "mw_interface-addWarning-2");
  println!("{:#?}", machine.register_library("mw_interface", table).unwrap());
  println!("{:#?}", machine.get_status().unwrap());
  println!("{:#?}", machine.cleanup_chunks(vec![]));
  println!("{:#?}", machine.load_file("@mw.lua", "mw.lua").unwrap());
  println!("{:#?}", machine.get_status().unwrap());
  println!("{:#?}", machine.call(4, LuaTable::default()).unwrap());
  //f.encode(ToLuaMessage::RegisterLibrary { name: "mw_interface".into(), functions: Default::default() }).unwrap();
  return;
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

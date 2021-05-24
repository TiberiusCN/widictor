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
  let converter = |text: Vec<Text>| -> String {
    fn convert_text(text: Vec<Text>, subwords: &mut HashSet<String>) -> String {
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
            let args: HashMap<String, String> = template.args.into_iter().map(|it| (it.0, convert_text(it.1, subwords))).collect();
            let mut com = template.com;
            if matches!(com[0], Text::Tab(1)) {
              com[0] = Text::Raw("#".to_string());
            }
            let com = convert_text(com, subwords);
            if com.starts_with("#") {
              out += format!("{{{}|{:?}}}", com, args).as_str();
            } else {
              println!("\x1b[32mT:{}\x1b[0m", com);
              let page = clean_raw(remote::get(&format!("Template:{}", com)).unwrap());
              // The entity {{{2L|Left}}} instructs the template to use the named parameter 2L or the text Left if the parameter is not present in the call. 
              let page = page.lines().fold(Vec::new(), |mut acc, it| {
                let mut tail = it;
                while !tail.is_empty() {
                  let (t, v) = Text::parse(tail, subwords).unwrap();
                  tail = t;
                  acc.push(v);
                }
                acc
              });
              out += &convert_text(page, subwords);
            }
          }
        }
      }
      out
    };
    convert_text(text, subwords)
  };
    
  let lang = lang.convert(converter);
  Ok(lang.subdivide())
}

pub struct Telua {
  pub machine: LuaInstance<std::process::ChildStdout, std::process::ChildStdin>,
}
impl Telua {
  pub fn new() -> Self {
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
    let init = machine.load_file("mwInit_lua", "mwInit.lua").unwrap().id;
    println!("{:#?}", machine.get_status().unwrap());
    let init = machine.call(init, LuaTable::default()).unwrap().result;
    println!("{:#?}", init);
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
    println!("{:#?}", machine.cleanup_chunks(init.iter().map(|(_, z)| z).copied().collect()));
    let mw_lua = machine.load_file("@mw.lua", "mw.lua").unwrap().id;
    println!("{:#?}", machine.get_status().unwrap());
    println!("{:#?}", machine.call(mw_lua, LuaTable::default()).unwrap().result);
    Self { machine }
  }
}

fn main() {
  let arg = std::env::args().nth(1).unwrap();
  scan(&arg);
}

fn clean_raw(src: String) -> String {
  let mut opened = false;
  let mut tag = String::new();
  let mut noinclude = false;
  let mut out = String::new();
  for c in src.chars() {
    match c {
      '<' => opened = true,
      '>' => {
        match tag.as_str() {
          "noinclude" => noinclude = true,
          "/noinclude" => noinclude = false,
          com if com.starts_with("!--") => {},
          _ => log::warn!("unknown tag: {}", tag),
        }
        tag.clear();
        opened = false
      },
      s if opened => tag.push(s),
      _ if noinclude => {},
      s => out.push(s),
    }
  }
  if let Some(redirect) = out.strip_prefix("#REDIRECT ") {
    let link = redirect.trim().strip_prefix("[[").unwrap().strip_suffix("]]").unwrap();
    clean_raw(remote::get(&link).unwrap())
  } else {
    out
  }
}
fn scan(word: &str) {
  let page = remote::get(word).map(|it| clean_raw(it)).unwrap();
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

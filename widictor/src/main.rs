use std::{collections::{HashMap, HashSet}, rc::Rc};
use language::Language;
use scribunto::*;
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

#[derive(serde::Deserialize)]
pub enum TypeId {
  Bool,
  Float,
  Integer,
  String,
}
#[derive(serde::Deserialize)]
pub struct Proto(HashMap<String, TypeId>);

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
            let args: HashMap<String, String> = template.args.into_iter().map(|it| (it.0, convert_text(it.1.1, subwords))).collect(); // ??? TODO
            let mut com = template.com;
            if matches!(com[0], Text::Tab(1)) {
              com[0] = Text::Raw("#".to_string());
            }
            let com = convert_text(com, subwords);
            if com.starts_with("#") {
              if let Some(module) = com.strip_prefix("# invoke:") {
                let mut telua = Telua::new();
                println!("\x1b[32mM:{}\x1b[0m", module);
                let proto: Proto = serde_json::from_reader(std::fs::File::open(format!("/tmp/widictor/modules/{}.proto", &module)).unwrap()).unwrap();
                let module = format!("/tmp/widictor/modules/{}.lua", module);
                let mut table = LuaTable::<LuaString>::default();
                for arg in args {
                  if let Some(tid) = proto.0.get(&arg.0) {
                    match tid {
                      TypeId::Bool => table.insert_bool(arg.0, arg.1 == "true"),
                      TypeId::Float => table.insert_float(arg.0, arg.1.parse::<f32>().unwrap()),
                      TypeId::Integer => table.insert_integer(arg.0, arg.1.parse::<i32>().unwrap()),
                      TypeId::String => table.insert_string(arg.0, &arg.1),
                    }
                  } else {
                    table.insert_string(arg.0, &arg.1)
                  }
                }
                let module = telua.machine.call_file(&module, &module).unwrap();
                out += format!("{{MODULE: {:?}}}", module).as_str();
                panic!("{}", out);
              } else {
                panic!("unknown: {}", com);
              }
              // #invoke:etymology/templates|inherited Module:…|function
            } else {
              match com.as_str() {
                "PAGENAME" => out += "PAGENAME",
                _ => {
                  if template.is_defval {
                    println!("\x1b[33mD: {} — {:?}", &com, &args);
                  } else {
                    println!("\x1b[32mT:{}\x1b[0m", com);
                    let file = format!("/tmp/widictor/{}", com);
                    let source = std::fs::read_to_string(file).unwrap();
                    //let page = clean_raw(remote::get(&format!("Template:{}", com)).unwrap());
                    let page = source;
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
          }
        }
      }
      out
    }
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
      "pkg/mw_main.lua",
      "pkg",
      0,
      4,
      vec!["pkg".to_owned()],
    ).unwrap();
    let machine = Self { machine };

    machine.machine.call_file("mwInit_lua", "mwInit.lua").unwrap();

    let mut table = LuaTable::<LuaString>::default();
    //table.insert_string("", "mw-require");
    table.insert_string("loadPackage", "loadPackage");
    machine.machine.insert_callback("loadPackage", Box::new(|instance: &mut LuaInstance<_, _>, table: LuaTable<LuaInteger>| {
      let file_id = table.get_string(1).unwrap().as_raw().to_owned();
      let file = if let Some(file_id) = file_id.strip_prefix("Module:") {
        //format!("/tmp/widictor/modules/{}.lua", file_id)
        format!("{}.lua", file_id)
      } else {
        let file_id = match file_id.as_str() {
          "ustring" => "ustring/ustring",
          u => u,
        };
        //format!("pkg/{}.lua", file_id)
        format!("{}.lua", file_id)
      };
      println!("req: \x1b[31m{}\x1b[0m", file);
      // let file = std::fs::read_to_string(file).unwrap();
      // let file = file.replace("\\", "\\\\")
      //   .replace("\n", "\\n")
      //   .replace("\r", "\\r")
      //   .replace("\"", "\\\"");
      let chunk = instance.load_file(&file_id, &file).unwrap();
      let mut out = LuaTable::default();
      out.insert_chunk(1, chunk);
      out
    }));
    // fakes
    [
      ("loadPHPLibrary", "mw_interface-loadPHPLibrary-2"),
      ("frameExists", "mw_interface-frameExists-2"),
      ("newChildFrame", "mw_interface-newChildFrame-2"),
      ("getExpandedArgument", "mw_interface-getExpandedArgument-2"),
      ("getAllExpandedArguments", "mw_interface-getAllExpandedArguments-2"),
      ("expandTemplate", "mw_interface-expandTemplate-2"),
      ("callParserFunction", "mw_interface-callParserFunction-2"),
      ("preprocess", "mw_interface-preprocess-2"),
      ("incrementExpensiveFunctionCount", "mw_interface-incrementExpensiveFunctionCount-2"),
      ("isSubsting", "mw_interface-isSubsting-2"),
      ("getFrameTitle", "mw_interface-getFrameTitle-2"),
      ("setTTL", "mw_interface-setTTL-2"),
      ("addWarning", "mw_interface-addWarning-2"),
      ].iter().for_each(|it| table.insert_string(it.0, it.1));

    //
    println!("mw_interface: {:#?}", machine.machine.register_library("mw_interface", table).unwrap());

    let mut table = LuaTable::<LuaString>::default();
    table.insert_string("require", "mw-require");
    println!("{:#?}", machine.machine.register_library("vm", table).unwrap());
    machine.machine.insert_callback("mw-require", Box::new(|_instance: &mut LuaInstance<_, _>, table: LuaTable<LuaInteger>| {
      let file_id = table.get_string(1).unwrap().as_raw().to_owned();
      let file = if let Some(file_id) = file_id.strip_prefix("Module:") {
        format!("/tmp/widictor/modules/{}.lua", file_id)
      } else {
        let file_id = match file_id.as_str() {
          "libraryUtil" => "libraryUtil",
          "ustring" => "ustring/ustring",
          "mw" => "mw",
          "mw.site" => "mw.site",
          "mw.uri" => "mw.uri",
          "mw.ustring" => "mw.ustring",
          "mw.language" => "mw.language",
          "mw.message" => "mw.message",
          "mw.title" => "mw.title",
          "mw.text" => "mw.text",
          "mw.html" => "mw.html",
          "mw.hash" => "mw.hash",
          e => panic!("unknown file: {}", e),
        };
        format!("pkg/{}.lua", file_id)
      };
      println!("req: \x1b[31m{}\x1b[0m", file);
      let file = std::fs::read_to_string(file).unwrap();
       let file = file.replace("\\", "\\\\")
         .replace("\n", "\\n")
         .replace("\r", "\\r")
         .replace("\"", "\\\"");
       let mut out = LuaTable::default();
       out.insert_string(1, file.as_str());
       out
    }));

    let setup_interface = |machine: Telua, name, arg| {
      let setup = machine.machine.call_file(name, &format!("{}.lua", name))?.get_string_table(1).and_then(|it| it.get_function("setupInterface")).unwrap();
      let mut args = LuaTable::default();
      args.insert_string_table(1, arg);
      machine.machine.call(setup, args)?;
      Ok(())
    };

    setup_interface(machine, "mw", {
      let mut args = LuaTable::default();
      args.insert_bool("allowEnvFuncs", false);
      args
    }).unwrap();
    machine.machine.load_file("package", "package.lua").unwrap();
    let mut table = LuaTable::default();
    // fakes
    [
      ("getNsIndex", "mw_interface-getNsIndex-3"),
      ("pagesInCategory", "mw_interface-pagesInCategory-3"),
      ("pagesInNamespace", "mw_interface-pagesInNamespace-3"),
      ("userInGroup", "mw_interface-userInGroup-3"),
      ("interwikiMap", "mw_interface-interwikiMap-3"),
      ].iter().for_each(|it| table.insert_string(it.0, it.1));
    println!("mw_interface: {:#?}", machine.machine.register_library("mw_interface", table).unwrap());

    setup_interface(machine, "mw.site", {
      let mut args = LuaTable::default();
      [
        ("siteName", "widictor"),
        ("server", "http://localhost"),
        ("scriptPath", ""),
        ("stylePath", ""),
        ("current_version", env!("CARGO_PKG_VERSION")),
      ].iter().for_each(|it| args.insert_string(it.0, it.1));
      args.insert_integer_table("namespaces", LuaTable::default());
      let stats = [
        ("pages", 1),
        ("articles", 0),
        ("files", 0),
        ("edits", 0),
        ("users", 1),
        ("activeUsers", 1),
        ("admins", 1),
      ].iter().fold(LuaTable::default(), |mut acc, it| {
        acc.insert_integer(it.0, it.1);
        acc
      });
      args.insert_string_table("stats", stats);
      args
    }).unwrap();
      ("mw.uri", &|_| {}),
      ("mw.ustring", &|args| {
        args.insert_integer("stringLengthLimit", 2097152);
        args.insert_integer("patternLengthLimit", 10000);
      }),
      ("mw.language", &|_| {}),
      ("mw.message", &|args| {
        args.insert_string("lang", "la");
      }),
      ("mw.title", &|args| {
        let mut this_title = LuaTable::default();
        this_title.insert_bool("isCurrentTitle", true);
        this_title.insert_bool("isLocal", true);
        this_title.insert_string("interwiki", "");
        this_title.insert_integer("namespace", 0);
        this_title.insert_string("nsText", "");
        this_title.insert_string("text", "Sample");
        this_title.insert_string("fragment", "");
        this_title.insert_string("thePartialUrl", "Sample");
        this_title.insert_bool("file", false);
        args.insert_string_table("thisTitle", this_title);
        args.insert_integer("NS_MEDIA", -2);
      }),
      ("mw.text", &|args| {
        args.insert_string_table("nowiki_protocols", LuaTable::default());
        args.insert_string("comma", ",");
        args.insert_string("and", " et ");
        args.insert_string("ellipsis", "...");
      }),
      ("mw.html", &|args| {
        args.insert_string("uniqPrefix", "^?'\"`UNIQ-");
        args.insert_string("uniqSuffix", "-QINU`\"'^?");
      }),
      ("mw.hash", &|_| {}),
    ];
    for it in z {
      if let Some(setup) = machine.call_file(it.0, &format!("{}.lua", it.0)).unwrap().get_string_table(1).and_then(|it| it.get_function("setupInterface")) {
        let mut table = LuaTable::default();
        it.1(&mut table);
        let mut args = LuaTable::default();
        args.insert_string_table(1, table);
        machine.call(setup, args).unwrap();
      } else {
        it.1(&mut Default::default());
      }
    }

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

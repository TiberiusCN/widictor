use crate::{remote, scribunto::*};
use language::Language;
use std::{
  collections::{HashMap, HashSet},
  rc::Rc,
};
use text::Text;
use word_section::WordSection;

mod language;
mod section;
pub(crate) mod substr;
mod text;
mod wiki_error;
mod word_section;

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
#[derive(Default, serde::Deserialize)]
pub struct Proto(HashMap<String, TypeId>);

enum AnyParse<'a> {
  Language(Language<(), WordSection<()>>),
  Section(WordSection<()>),
  Content(&'a str),
}
impl<'a> AnyParse<'a> {
  fn parse(input: &'a str) -> Self {
    if let Ok(r) = Language::parse(input).map(|v| v.1).map(Self::Language) {
      r
    } else if let Ok(r) = WordSection::parse(input).map(|v| v.1).map(Self::Section) {
      r
    } else {
      Self::Content(input)
    }
  }
}

fn parse_page(
  page: &str,
  language: &str,
  subwords: &mut HashSet<String>,
) -> Result<Vec<Language<String, Rc<WordSection<String>>>>, ()> {
  let mut iter = page.lines();
  let mut lang = {
    let iter = &mut iter;
    (move || {
      while let Some(line) = iter.next() {
        match Language::parse(line) {
          Ok(lang) if lang.1.name == language => return Ok(lang.1),
          Ok(_) => {}
          Err(nom::Err::Error(e)) if !e.filtered() => return Err(()),
          Err(_) => {}
        }
      }
      Err(())
    })()?
  }
  .convert(|_| unreachable!());
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
    let mut telua = Telua::new().unwrap();
    fn convert_text(text: Vec<Text>, subwords: &mut HashSet<String>, telua: &mut Telua, frame: &Frame) -> String {
      let mut out = String::new();
      for text in text {
        if !out.is_empty() {
          out += " ";
        }
        match text {
          Text::Raw(raw) => out += raw.as_str(),
          Text::Tab(tab) => {
            for _ in (0..).take(tab as usize) {
              out += "»";
            }
          }
          Text::Template(template) => {
            let args: HashMap<String, String> =
              template.args.into_iter().map(|it| (it.0, convert_text(it.1 .1, subwords, telua, frame))).collect(); // ??? TODO
            let mut com = template.com;
            if matches!(com[0], Text::Tab(1)) {
              com[0] = Text::Raw("#".to_string());
            }
            let com = convert_text(com, subwords, telua, frame);
            if com.starts_with("#") {
              if let Some(module) = com.strip_prefix("# invoke:") {
                let mut i = module.splitn(2, ":");
                let module = i.next().unwrap();
                let function = i.next().unwrap();
                println!("\x1b[32mM:{}\x1b[0m", module);
                let proto: Proto = serde_json::from_reader(
                  std::fs::File::open(format!("/tmp/widictor/modules/{}.proto", &module)).unwrap(),
                )
                .unwrap();
                let frame = telua.new_frame(args, proto, Some(&frame)).unwrap();
                let _module = telua.call(&module, &function, frame).unwrap();
                //out += format!("{{MODULE: {:#?}}}", module).as_str();
                //panic!("{}", out);
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
                    out += &convert_text(page, subwords, telua, frame);
                  }
                }
              }
            }
          }
        }
      }
      out
    }
    let frame = telua.new_frame(Default::default(), Default::default(), None).unwrap();
    convert_text(text, subwords, &mut telua, &frame)
  };

  let lang = lang.convert(converter);
  Ok(lang.subdivide())
}

pub struct Telua {
  pub machine: LuaInstance<std::process::ChildStdout, std::process::ChildStdin>,
  libs: HashMap<&'static str, LuaTable<LuaString>>,
}
type TeluaError = Box<dyn std::error::Error>;
type TeluaResult<T> = Result<T, TeluaError>;
type ApiMap = HashMap<
  &'static str,
  Box<
    dyn Fn(
      &mut LuaInstance<std::process::ChildStdout, std::process::ChildStdin>,
      LuaTable<LuaInteger>,
    ) -> LuaTable<LuaInteger>,
  >,
>;
impl Telua {
  fn empty() -> TeluaResult<Self> {
    let machine = LuaInstance::new(
      "pkg/mw_main.lua",
      "pkg",
      0,
      4,
      vec!["pkg".to_owned(), "pkg/ustring".to_owned(), "/tmp/widictor/modules".to_owned()],
    )?;
    Ok(Self { machine, libs: Default::default() })
  }
  fn mw_init(&mut self) -> TeluaResult<()> {
    self.machine.call_file("mwInit_lua", "mwInit.lua")?;
    Ok(())
  }
  fn package(&mut self) -> TeluaResult<()> {
    self.machine.load_file("package", "package.lua")?;
    Ok(())
  }
  fn register_library(&mut self, libname: &str, version: u32, api: ApiMap) -> TeluaResult<()> {
    let mut table = LuaTable::default();
    for (id, l) in api {
      let name = format!("{}-{}-{}", libname, id, version);
      table.insert_string(id, &name);
      self.machine.insert_callback(&name, l);
    }
    self.machine.register_library(libname, table)?;
    Ok(())
  }
  fn mw_interface_1(&mut self) -> TeluaResult<()> {
    let api = ApiMap::new();
    self.register_library("mw_interface", 1, api)
  }
  fn mw_interface_2(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert(
      "loadPackage",
      Box::new(|instance, args| {
        let file_id = args.get_string(1).unwrap().as_raw().to_owned();
        let file = if let Some(file_id) = file_id.strip_prefix("Module:") {
          format!("{}.lua", file_id)
        } else {
          let file_id = match file_id.as_str() {
            "ustring" => "ustring/ustring",
            u => u,
          };
          format!("{}.lua", file_id)
        };
        println!("req: \x1b[31m{}\x1b[0m", file);
        let chunk = instance.load_file(&file_id, &file).unwrap();
        let mut out = LuaTable::default();
        out.insert_chunk(1, chunk);
        out
      }),
    );
    api.insert(
      "loadPHPLibrary",
      Box::new(|instance, args| {
        let file_id = args.get_string(1).unwrap().as_raw().to_owned();
        let file = if let Some(id) = file_id.strip_prefix("Module:") { id.to_owned() } else { file_id };
        println!("reqphp: \x1b[33m{}\x1b[0m", file);
        let api = instance.call_file(&file, &format!("{}.lua", &file)).unwrap();
        let api = if let Some(mut api) = api.get_string_table(1) {
          let mut old_api = Default::default();
          std::mem::swap(&mut api.value, &mut old_api);
          api.value = old_api
            .into_iter()
            .map(|(name, val)| {
              if let Some(val) = val.as_string_table() {
                if let Some(id) = val.object.as_ref() {
                  if id == "Scribunto_LuaStandaloneInterpreterFunction" {
                    let f = val.get_integer("id").unwrap();
                    return (name, Box::new(f.to_chunk().into()));
                  }
                }
              }
              (name, val)
            })
            .collect();

          let mut wrap = LuaTable::default();
          wrap.insert_string_table(1, api);
          wrap
        } else {
          api
        };
        api
      }),
    );
    api.insert("frameExists", Box::new(|_, _| todo!()));
    api.insert("newChildFrame", Box::new(|_, _| todo!()));
    api.insert("getExpandedArgument", Box::new(|_, _| todo!()));
    api.insert("getAllExpandedArguments", Box::new(|_, _| todo!()));
    api.insert("expandTemplate", Box::new(|_, _| todo!()));
    api.insert("callParserFunction", Box::new(|_, _| todo!()));
    api.insert("preprocess", Box::new(|_, _| todo!()));
    api.insert("incrementExpensiveFunctionCount", Box::new(|_, _| todo!()));
    api.insert("isSubstring", Box::new(|_, _| todo!()));
    api.insert("getFrameTitle", Box::new(|_, _| todo!()));
    api.insert("setTTL", Box::new(|_, _| todo!()));
    api.insert("addWarning", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 2, api)
  }
  fn mw_interface_3(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("getNsIndex", Box::new(|_, _| todo!()));
    api.insert("pagesInCategory", Box::new(|_, _| todo!()));
    api.insert("pagesInNamespace", Box::new(|_, _| todo!()));
    api.insert("userInGroup", Box::new(|_, _| todo!()));
    api.insert("interwikiMap", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 3, api)
  }
  fn mw_interface_4(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("anchorEncode", Box::new(|_, _| todo!()));
    api.insert("localUrl", Box::new(|_, _| todo!()));
    api.insert("fullUrl", Box::new(|_, _| todo!()));
    api.insert("canonicalUrl", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 4, api)
  }
  fn mw_interface_5(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("find", Box::new(|_, _| todo!()));
    api.insert("match", Box::new(|_, _| todo!()));
    api.insert("gmatch_init", Box::new(|_, _| todo!()));
    api.insert("gmatch_callback", Box::new(|_, _| todo!()));
    api.insert("gsub", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 5, api)
  }
  fn mw_interface_6(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert(
      "getContLangCode",
      Box::new(|_, _| {
        let mut ret = LuaTable::default();
        ret.insert_string(1, "la");
        ret
      }),
    );
    api.insert("isSupportedLanguage", Box::new(|_, _| todo!()));
    api.insert("isKnownLanguageTag", Box::new(|_, _| todo!()));
    api.insert("isValidCode", Box::new(|_, _| todo!()));
    api.insert("isValidBuiltInCode", Box::new(|_, _| todo!()));
    api.insert("fetchLanguageName", Box::new(|_, _| todo!()));
    api.insert("fetchLanguageNames", Box::new(|_, _| todo!()));
    api.insert("getFallbacksFor", Box::new(|_, _| todo!()));
    api.insert("lcfirst", Box::new(|_, _| todo!()));
    api.insert("ucfirst", Box::new(|_, _| todo!()));
    api.insert("lc", Box::new(|_, _| todo!()));
    api.insert("uc", Box::new(|_, _| todo!()));
    api.insert("caseFold", Box::new(|_, _| todo!()));
    api.insert("formatNum", Box::new(|_, _| todo!()));
    api.insert("formatDate", Box::new(|_, _| todo!()));
    api.insert("formatDuration", Box::new(|_, _| todo!()));
    api.insert("getDurationIntervals", Box::new(|_, _| todo!()));
    api.insert("parseFormattedNumber", Box::new(|_, _| todo!()));
    api.insert("convertPlural", Box::new(|_, _| todo!()));
    api.insert("convertGrammar", Box::new(|_, _| todo!()));
    api.insert("gender", Box::new(|_, _| todo!()));
    api.insert("isRTL", Box::new(|_, _| todo!()));
    api.insert("find", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 6, api)
  }
  fn mw_interface_7(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("plain", Box::new(|_, _| todo!()));
    api.insert("check", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 7, api)
  }
  fn mw_interface_8(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("newTitle", Box::new(|_, _| todo!()));
    api.insert("makeTitle", Box::new(|_, _| todo!()));
    api.insert("getExpensiveData", Box::new(|_, _| todo!()));
    api.insert("getUrl", Box::new(|_, _| todo!()));
    api.insert("getContent", Box::new(|_, _| todo!()));
    api.insert("getFileInfo", Box::new(|_, _| todo!()));
    api.insert("protectionLevels", Box::new(|_, _| todo!()));
    api.insert("cascadingProtection", Box::new(|_, _| todo!()));
    api.insert("redirectTarget", Box::new(|_, _| todo!()));
    api.insert("recordVaryFlag", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 8, api)
  }
  fn mw_interface_9(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("unstrip", Box::new(|_, _| todo!()));
    api.insert("unstripNoWiki", Box::new(|_, _| todo!()));
    api.insert("killMarkers", Box::new(|_, _| todo!()));
    api.insert("getEntityTable", Box::new(|_, _| todo!()));
    api.insert("jsonEncode", Box::new(|_, _| todo!()));
    api.insert("jsonDecode", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 9, api)
  }
  fn mw_interface_10(&mut self) -> TeluaResult<()> {
    let api = ApiMap::new();
    self.register_library("mw_interface", 10, api)
  }
  fn mw_interface_11(&mut self) -> TeluaResult<()> {
    let mut api = ApiMap::new();
    api.insert("listAlgorithms", Box::new(|_, _| todo!()));
    api.insert("hashValue", Box::new(|_, _| todo!()));
    self.register_library("mw_interface", 11, api)
  }
  fn setup_interface<F: Fn(&mut LuaTable<LuaString>)>(&mut self, name: &'static str, arg_gen: F) -> TeluaResult<()> {
    let lib = self.machine.call_file(name, &format!("{}.lua", name))?.get_string_table(1).unwrap();
    let setup = lib.get_function("setupInterface").ok_or_else(|| format!("setupInterface not found for {}", name))?;
    let mut args = LuaTable::default();
    args.insert_string_table(1, {
      let mut args = LuaTable::default();
      arg_gen(&mut args);
      args
    });
    self.machine.call(setup, args)?;
    self.libs.insert(name, lib);
    Ok(())
  }
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let mut machine = Self::empty()?;
    machine.mw_interface_1()?;
    machine.mw_init()?;
    machine.setup_interface("mw.frame", |_| {})?;
    machine.mw_interface_2()?;
    machine.setup_interface("mw", |it| {
      it.insert_bool("allowEnvFuncs", false);
    })?;
    machine.package()?;
    machine.mw_interface_3()?;
    machine.setup_interface("mw.site", |it| {
      [
        ("siteName", "widictor"),
        ("server", "http://localhost"),
        ("scriptPath", ""),
        ("stylePath", ""),
        ("current_version", env!("CARGO_PKG_VERSION")),
      ]
      .iter()
      .for_each(|f| it.insert_string(f.0, f.1));
      it.insert_integer_table("namespaces", LuaTable::default());
      let stats =
        [("pages", 1), ("articles", 0), ("files", 0), ("edits", 0), ("users", 1), ("activeUsers", 1), ("admins", 1)]
          .iter()
          .fold(LuaTable::default(), |mut acc, it| {
            acc.insert_integer(it.0, it.1);
            acc
          });
      it.insert_string_table("stats", stats);
    })?;
    machine.mw_interface_4()?;
    machine.setup_interface("mw.uri", |_| {})?;
    machine.mw_interface_5()?;
    machine.setup_interface("mw.ustring", |it| {
      it.insert_integer("stringLengthLimit", 2097152);
      it.insert_integer("patternLengthLimit", 10000);
    })?;
    machine.mw_interface_6()?;
    machine.setup_interface("mw.language", |_| {})?;
    machine.mw_interface_7()?;
    machine
      .setup_interface("mw.message", |it| {
        it.insert_string("lang", "la");
      })
      .unwrap();
    machine.mw_interface_8()?;
    machine.setup_interface("mw.title", |it| {
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
      it.insert_string_table("thisTitle", this_title);
      it.insert_integer("NS_MEDIA", -2);
    })?;
    machine.mw_interface_9()?;
    machine.setup_interface("mw.text", |it| {
      it.insert_string_table("nowiki_protocols", LuaTable::default());
      it.insert_string("comma", ",");
      it.insert_string("and", " et ");
      it.insert_string("ellipsis", "...");
    })?;
    machine.mw_interface_10()?;
    machine.setup_interface("mw.html", |it| {
      // it.insert_string("uniqPrefix", "^?'\"`UNIQ-");
      // it.insert_string("uniqSuffix", "-QINU`\"'^?");
      it.insert_string("uniqPrefix", "UNIQ-");
      it.insert_string("uniqSuffix", "-QINU");
    })?;
    machine.mw_interface_11()?;
    machine.setup_interface("mw.hash", |_| {})?;

    Ok(machine)
  }
  pub fn call(&mut self, file: &str, function: &str, frame: Frame) -> TeluaResult<String> {
    let chunk = self.machine.load_file(file, &format!("{}.lua", file))?;
    let mw = self.libs.get("mw").unwrap();
    let execute_module = mw.get_function("executeModule").unwrap();
    let mut args = LuaTable::default();
    args.insert_chunk(1, chunk);
    args.insert_string(2, function);
    args.insert_string_table(3, frame.clone().into_raw());
    let out = self.machine.call(execute_module, args)?;
    let out = out.get_function(2).unwrap();
    let mut args = LuaTable::default();
    args.insert_string_table(1, frame.into_raw());
    let out = self.machine.call(out, args)?;
    panic!("{:#?}", out);

    // let table = self.machine.call_file(file, &format!("{}.lua", file))?;
    // let table = table.get_string_table(1).unwrap();
    // let function = table.get_function(function).unwrap();
    // let mut table = LuaTable::default();
    // table.insert_string_table(1, frame);
    // let out = self.machine.call(function, table)?;
    // panic!("{:#?}", out);
  }
  pub fn new_frame(
    &mut self,
    args: HashMap<String, String>,
    proto: Proto,
    parent: Option<&Frame>,
  ) -> TeluaResult<Frame> {
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
    if let Some(frame) = parent {
      let lambda = self.libs.get("mw.frame").unwrap().get_function("getParentFrame").unwrap();
      Ok(frame.child(table, lambda))
    } else {
      Ok(Frame::new(table))
    }
  }
}

#[derive(Clone)]
pub struct Frame(LuaTable<LuaString>);
impl Frame {
  pub fn new(args: LuaTable<LuaString>) -> Self {
    let mut table = LuaTable::default();
    table.insert_string_table("args", args);
    Self(table)
  }
  pub fn child(&self, args: LuaTable<LuaString>, lambda: LuaChunk) -> Self {
    let mut child = Self::new(args);
    child.0.insert_string_table("parent", self.0.clone());
    child.0.insert_chunk("getParent", lambda);
    child
  }
  pub fn into_raw(self) -> LuaTable<LuaString> {
    self.into()
  }
}
impl From<Frame> for LuaTable<LuaString> {
  fn from(src: Frame) -> Self {
    src.0
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
          com if com.starts_with("!--") => {}
          _ => log::warn!("unknown tag: {}", tag),
        }
        tag.clear();
        opened = false
      }
      s if opened => tag.push(s),
      _ if noinclude => {}
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

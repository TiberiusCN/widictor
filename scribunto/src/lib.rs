pub use lua_bool::LuaBool;
pub use lua_float::LuaFloat;
pub use lua_integer::LuaInteger;
pub use lua_null::LuaNull;
pub use lua_string::LuaString;
use lua_table::AnyLua;
pub use lua_table::LuaTable;
use nom::{IResult, bytes::complete::{tag, take_while1}};
use std::{collections::HashMap, fmt::Display, io::{Read, Write}, path::PathBuf, process::{ChildStdin, ChildStdout}, sync::Arc};

mod php_error;
mod lua_string;
mod lua_integer;
mod lua_float;
mod lua_table;
mod lua_bool;
mod lua_null;
use php_error::PhpError;

#[macro_export]
macro_rules! transparent_lua {
  ($wrap:ty, $raw:ty) => {
    impl From<$raw> for $wrap {
      fn from(s: $raw) -> Self {
        Self(s)
      }
    }
    impl From<$wrap> for $raw {
      fn from(s: $wrap) -> Self {
        s.0
      }
    }
    impl From<&$wrap> for $raw {
      fn from(s: &$wrap) -> Self {
        s.0.clone()
      }
    }
  };
}

pub struct LuaSender<W: Write> {
  writer: W,
}
impl<W: Write> From<W> for LuaSender<W> {
  fn from(writer: W) -> Self {
    Self {
      writer,
    }
  }
}
impl<W: Write> LuaSender<W> {
  pub fn encode(&mut self, message: ToLuaMessage) -> Result<(), std::io::Error> {
    let message = format!("{}", LuaTable::from(message));
    let length = message.len();
    write!(self.writer, "{:08x}", length)?;
    write!(self.writer, "{:08x}", length * 2 - 1)?;
    write!(self.writer, "{}", message)?;
    Ok(())
  }
}
pub struct LuaReceiver<R: Read> {
  reader: R,
}
impl<R: Read> From<R> for LuaReceiver<R> {
  fn from(reader: R) -> Self {
    Self {
      reader,
    }
  }
}
enum LuaResult {
  Ret(LuaTable<LuaInteger>),
  Call(LuaString, LuaTable<LuaInteger>),
}
impl<R: Read> LuaReceiver<R> {
  fn hex_u32_decode(src: &[u8]) -> Result<u32, Box<dyn std::error::Error>> {
    let raw = hex::decode(src)?;
    assert_eq!(raw.len(), 4);
    Ok(u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]))
  }
  fn decode(&mut self) -> Result<LuaResult, Box<dyn std::error::Error>> {
    let buf = &mut [0u8; 8];
    self.reader.read_exact(buf)?;
    let length: u32 = Self::hex_u32_decode(buf)?;
    self.reader.read_exact(buf)?;
    let test: u32 = Self::hex_u32_decode(buf)?;
    assert_eq!(length * 2 - 1, test);
    let mut buf = unsafe {
      let length = length as usize;
      let mut s = Vec::with_capacity(length);
      s.set_len(length);
      s
    };
    self.reader.read_exact(&mut buf)?;
    let table = std::str::from_utf8(buf.as_slice())?;
    let table = table.replace("\\\\", "\\");
    let (tail, table): (&str, LuaTable<LuaString>) = LuaTable::parse(&table).unwrap();
    assert!(tail.is_empty());
    let op = table.get_string("op").unwrap();
    match op.as_raw() {
      "return" => {
        let values = table.get_integer_table("values").unwrap();
        let nvalues = table.get_integer("nvalues").unwrap();
        assert_eq!(*nvalues.as_raw(), values.len() as i32);
        Ok(LuaResult::Ret(values.clone()))
      },
      "error" => {
        let err = table.get_string("value").unwrap();
        Err(Box::new(PhpError::<&str>::Lua(err.as_raw().to_owned()).into_nom()))
      },
      "call" => {
        let args = table.get_integer_table("args").unwrap();
        let nargs = table.get_integer("nargs").unwrap();
        let id = table.get_string("id").unwrap();
        assert_eq!(*nargs.as_raw(), args.len() as i32);
        Ok(LuaResult::Call(id.clone(), args.clone()))
      }
      s => Err(Box::new(PhpError::<&str>::UnknownOp(s.to_owned()).into_nom())),
    }
  }
}

#[derive(Debug)]
pub struct RGetStatus {
  pub pid: u32,
  // user + system in clock ticks
  pub time: u32,
  // virtual memory size in octets
  pub vsize: u32,
  // // resident set size in octets
  // pub rss: u32,
}
#[derive(Debug)]
pub struct RLoadString {
  pub id: i32,
}
#[derive(Debug)]
pub struct RCallLuaFunction {
  pub result: HashMap<String, i32>,
}
#[derive(Debug)]
pub struct RRegisterLibrary {}
#[derive(Debug)]
pub struct RCleanupChunks {}

pub struct LuaInstance<R: Read, W: Write> {
  input: LuaReceiver<R>,
  output: LuaSender<W>,
  includes: Vec<PathBuf>,
  library: HashMap<LuaString, Arc<Box<dyn Fn(&mut LuaInstance<R, W>, LuaTable<LuaInteger>) -> LuaTable<LuaString>>>>,
}
impl<R: Read, W: Write> LuaInstance<R, W> {
  fn decode_ack(&mut self, src: LuaResult) -> Result<LuaTable<LuaInteger>, Box<dyn std::error::Error>> {
    match src {
      LuaResult::Ret(ret) => {
        println!("ret");
        Ok(ret)
      },
      LuaResult::Call(id, args) => {
        println!("call {}", id);
        if let Some(l) = self.library.get(&id) {
          let l = l.clone();
          let result = l(self, args);
          self.output.encode(ToLuaMessage::Return { values: result })?;
          let r = self.input.decode()?;
          self.decode_ack(r)
        } else {
          Err(Box::new(PhpError::<&str>::NoSuchFunction(id.as_raw().to_owned())))
        }
      }
    }
  }
  pub fn weld(input: LuaReceiver<R>, output: LuaSender<W>, includes: Vec<String>) -> Self {
    let includes = includes.into_iter().map(Into::into).collect();
    Self { input, output, includes, library: HashMap::new() }
  }
  pub fn insert_callback(&mut self, op: &str, lambda: Box<dyn Fn(&mut LuaInstance<R, W>, LuaTable<LuaInteger>) -> LuaTable<LuaString>>) {
    self.library.insert(op.into(), Arc::new(lambda));
  }
  pub fn get_status(&mut self) -> Result<RGetStatus, Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::GetStatus)?;
    let r = self.input.decode()?;
    let r = self.decode_ack(r)?;
    let r = r.get_string_table(1).unwrap();
    Ok(RGetStatus {
      pid: *r.get_integer("pid").unwrap().as_raw() as _,
      time: *r.get_integer("time").unwrap().as_raw() as _,
      vsize: *r.get_integer("vsize").unwrap().as_raw() as _,
    })
  }
  ///// untested
  pub fn load_string(&mut self, name: &str, text: &str) -> Result<RLoadString, Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::LoadString { text: text.into(), name: name.into() })?;
    let r = self.input.decode()?;
    let r = self.decode_ack(r)?;
    Ok(RLoadString {
      id: *r.get_integer(1).unwrap().as_raw(),
    })
  }
  pub fn load_file(&mut self, name: &str, file: &str) -> Result<RLoadString, Box<dyn std::error::Error>> {
    for p in self.includes.iter() {
      let p = p.join(file);
      if p.exists() {
        let file = std::fs::read_to_string(p)?;
        let file = file.replace("\\", "\\\\")
          .replace("\n", "\\n")
          .replace("\r", "\\r")
          .replace("\"", "\\\"");
        return self.load_string(name, &file);
      }
    }
    Err(format!("file {} not found", file).into())
  }
  pub fn call(&mut self, id: i32, args: LuaTable<LuaString>) -> Result<RCallLuaFunction, Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::Call { id: id.into(), args })?;
    let r = self.input.decode()?;
    let r = self.decode_ack(r)?;
    if let Some(z) = r.get_string_table(1) {
      let result = z.as_ref().iter().map(|(func, op)| {
        (func.as_raw().to_string(), op.as_string_table().unwrap().get_integer("id").unwrap().as_raw().clone())
      }).collect();
      Ok(RCallLuaFunction {
        result,
      })
    } else {
      Ok(RCallLuaFunction {
        result: Default::default(),
      })
    }
  }
  pub fn register_library(&mut self, name: &str, functions: LuaTable<LuaString>) -> Result<RRegisterLibrary, Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::RegisterLibrary { name: name.into(), functions })?;
    let _ = self.input.decode()?;
    Ok(RRegisterLibrary {})
  }
  pub fn cleanup_chunks(&mut self, owned: Vec<i32>) -> Result<RCleanupChunks, Box<dyn std::error::Error>> {
    let mut ids = LuaTable {
      value: Default::default(),
      object: None,
    };
    for id in owned.into_iter() {
      ids.insert_bool(id as i32, true);
    }
    self.output.encode(ToLuaMessage::CleanupChunks { ids })?;
    let _ = self.input.decode()?;
    Ok(RCleanupChunks {})
  }
  pub fn quit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::Quit)?;
    Ok(())
  }
  pub fn test_quit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    self.output.encode(ToLuaMessage::Testquit)?;
    Ok(())
  }
}
impl LuaInstance<ChildStdout, ChildStdin> {
  pub fn new(main: &str, includes: &str, interpreter_id: usize, int_size: usize, paths: Vec<String>) -> Result<Self, std::io::Error> {
    let path = paths.iter().fold("?.lua".to_owned(), |acc, it| format!("{};{}/?.lua", acc, it));
    let mut proc = std::process::Command::new("lua5.1")
      .arg(main)
      .arg(includes)
      .arg(format!("{}", interpreter_id).as_str())
      .arg(format!("{}", int_size).as_str())
      .env("LUA_PATH", path)
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .spawn()?;
    let input: LuaReceiver<_> = proc.stdout.take().unwrap().into();
    let output: LuaSender<_> = proc.stdin.take().unwrap().into();
    Ok(Self::weld(input, output, paths))
  }
}

struct Parser;
impl Parser {
  fn null_val(src: &str) -> IResult<&str, (), PhpError<&str>> {
    let (src, _) = tag("N")(src)?;
    Ok((src, ()))
  }
  fn close(src: &str) -> IResult<&str, (), PhpError<&str>> {
    let (src, _) = tag("}")(src)?;
    Ok((src, ()))
  }
  fn open(src: &str) -> IResult<&str, (), PhpError<&str>> {
    let (src, _) = tag("{")(src)?;
    Ok((src, ()))
  }
  fn separator(src: &str) -> IResult<&str, (), PhpError<&str>> {
    let (src, _) = tag(":")(src)?;
    Ok((src, ()))
  }
  fn prefix(src: &str) -> IResult<&str, &str, PhpError<&str>> {
    let (src, value) = take_while1(|s: char| s.is_ascii_alphanumeric() || s == '_')(src)?;
    let (src, _) = Self::separator(src)?;
    Ok((src, value))
  }
  fn usize_val(src: &str) -> IResult<&str, usize, PhpError<&str>> {
    let (src, val) = take_while1(|s: char| s.is_numeric())(src)?;
    let val: usize = val.parse().map_err(PhpError::from)?;
    Ok((src, val))
  }
  fn i32_val(src: &str) -> IResult<&str, i32, PhpError<&str>> {
    let (src, val) = take_while1(|s: char| s.is_numeric() || s == '-')(src)?;
    let val: i32 = val.parse().unwrap();
    Ok((src, val))
  }
  fn f32_val(src: &str) -> IResult<&str, f32, PhpError<&str>> {
    tag::<_, _, PhpError<&str>>("INF")(src).map(|(src, _): (&str, &str)| (src, f32::INFINITY))
      .or_else(|_| tag::<_, _, PhpError<&str>>("-INF")(src).map(|(src, _)| (src, f32::NEG_INFINITY)))
      .or_else(|_| tag::<_, _, PhpError<&str>>("NAN")(src).map(|(src, _)| (src, f32::NAN)))
      .or_else(|_| {
        let (src, val) = take_while1(|s: char| s.is_numeric() || s == '-' || s ==',' || s == '.')(src)?;
        let val: f32 = val.replace(',', ".").parse().map_err(PhpError::from)?;
        Ok((src, val))
      })
  }
  fn str_val(src: &str, len: usize) -> IResult<&str, String, PhpError<&str>> {
    let (src, _) = tag("\"")(src)?;
    if src.len() < len { return Err(PhpError::BadLength(len as _, src.len() as _).into()); }
    let out = &src[0..len].to_owned();
    let src = &src[len..];
    let (src, _) = tag("\"")(src)?;
    let mut val = String::new();
    let mut slash = false;
    for c in out.chars() {
      if slash {
        val.push(match c {
          't' => '\t',
          'r' => '\r',
          'n' => '\n',
          c => c,
        });
        slash = false;
      } else {
        if c == '\\' {
          slash = true;
        } else {
          val.push(c);
        }
      }
    }
    Ok((src, val))
  }
  fn finite(src: &str) -> IResult<&str, (), PhpError<&str>> {
    tag(";")(src).map(|(src, _)| (src, ()))
  }
  fn any_lua(src: &str) -> IResult<&str, AnyLua, PhpError<&str>> {
    LuaString::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) })
      .or_else(|_| LuaBool::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
      .or_else(|_| LuaNull::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
      .or_else(|_| LuaInteger::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
      .or_else(|_| LuaFloat::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
      .or_else(|_| LuaTable::<LuaInteger>::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
      .or_else(|_| LuaTable::<LuaString>::parse(src).map(|(a, b)| -> (&str, AnyLua) { (a, b.into()) }))
  }
}

pub trait LuaType: 'static + Display + std::fmt::Debug + Clone {}
pub trait LuaNameType: LuaType + Eq + std::hash::Hash {
  fn try_from_string(src: LuaString) -> Result<Box<Self>, LuaString>;
  fn try_from_integer(src: LuaInteger) -> Result<Box<Self>, LuaInteger>;
}

pub enum ToLuaMessage {
  LoadString { text: LuaString, name: LuaString },
  Call { id: LuaInteger, args: LuaTable<LuaString> },
  RegisterLibrary { name: LuaString, functions: LuaTable<LuaString> },
  GetStatus,
  CleanupChunks { ids: LuaTable<LuaInteger> },
  Quit,
  Testquit,
  Return { values: LuaTable<LuaString> },
  Failure { value: LuaString },
}
impl From<ToLuaMessage> for LuaTable<LuaString> {
  fn from(src: ToLuaMessage) -> Self {
    let mut t = Self::default();
    match src {
      ToLuaMessage::LoadString { text, name } => {
        t.insert_string("op", "loadString");
        t.insert_string("text", text);
        t.insert_string("chunkName", name);
      }
      ToLuaMessage::Call { id, args } => {
        t.insert_string("op", "call");
        t.insert_integer("id", id);
        t.insert_integer("nargs", args.as_ref().len() as i32);
        t.insert_string_table("args", args);
      }
      ToLuaMessage::RegisterLibrary { name, functions } => {
        t.insert_string("op", "registerLibrary");
        t.insert_string("name", name);
        t.insert_string_table("functions", LuaTable::from(functions));
      }
      ToLuaMessage::GetStatus => {
        t.insert_string("op", "getStatus");
      }
      ToLuaMessage::CleanupChunks { ids } => {
        t.insert_string("op", "cleanupChunks");
        t.insert_integer_table("ids", ids);
      }
      ToLuaMessage::Quit => {
        t.insert_string("op", "quit");
      }
      ToLuaMessage::Testquit => {
        t.insert_string("op", "testquit");
      }
      ToLuaMessage::Return { values } => {
        t.insert_string("op", "return");
        t.insert_integer("nvalues", values.len() as i32);
        t.insert_string_table("values", values);
      }
      ToLuaMessage::Failure { value } => {
        t.insert_string("op", "error");
        t.insert_string("value", value);
      },
    }
    t
  }
}

pub enum FromLuaMessage {
  Call { id: LuaInteger, args: LuaTable<LuaString> },
}

pub struct LuaResponse {
  op: String,
  values: LuaTable<LuaInteger>,
}
#[derive(Debug)]
pub struct LuaFailed {
  value: LuaString,
  trace: LuaTable<LuaString>,
}
impl Display for LuaFailed {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value.as_raw())
  }
}

#[test]
fn test() {
  let (last, val) = LuaString::parse(r#"s:6:"A to Z";"#).unwrap();
  assert!(last.is_empty());
  assert_eq!(val, LuaString::from("A to Z"));
  let (last, val) = LuaFloat::parse(r"d:-1.23;").unwrap();
  assert!(last.is_empty());
  assert_eq!(f32::from(val), -1.23);
  let (last, val) = LuaInteger::parse(r"i:-882;").unwrap();
  assert!(last.is_empty());
  assert_eq!(i32::from(val), -882);
  let (last, val) = LuaBool::parse(r"b:0;").unwrap();
  assert!(last.is_empty());
  assert_eq!(bool::from(val), false);
  let (last, val) = LuaBool::parse(r"b:1;").unwrap();
  assert!(last.is_empty());
  assert_eq!(bool::from(val), true);
  let (last, _) = LuaNull::parse(r"N;").unwrap();
  assert!(last.is_empty());
  let (last, val): (_, LuaTable<LuaInteger>) = LuaTable::parse(r#"a:4:{i:0;b:1;i:1;N;i:2;d:-421000000;i:3;s:6:"A to Z";}"#).unwrap();
  assert!(last.is_empty());
  assert!(val.object.is_none());
  {
    assert_eq!(bool::from(val.get_bool(0).unwrap()), true);
    assert!(val.get_null(1).is_some());
    assert_eq!(f32::from(val.get_float(2).unwrap()), -421000000.0);
    assert_eq!(val.get_string(3).unwrap().as_ref(), "A to Z");
  }
  let (last, val): (_, LuaTable<LuaString>) = LuaTable::parse(r#"a:2:{i:42;b:1;s:6:"A to Z";a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}}"#).unwrap();
  assert!(last.is_empty());
  assert!(val.object.is_none());
  {
    assert_eq!(bool::from(val.get_bool("42").unwrap()), true);
    let val = val.get_integer_table("A to Z").unwrap();
    for i in 0..=2 {
      assert_eq!(i32::from(val.get_integer(i).unwrap()), i+1);
    }
  }
  let (last, val): (_, LuaTable<LuaString>) = LuaTable::parse(r#"O:8:"stdClass":2:{s:4:"John";d:3.14;s:4:"Jane";d:2.718;}"#).unwrap();
  assert!(last.is_empty());
  assert!(val.object.as_ref().map(|v| String::from(v.clone())) == Some("stdClass".to_owned()));
  {
    assert_eq!(f32::from(val.get_float("John").unwrap()), 3.14);
    assert_eq!(f32::from(val.get_float("Jane").unwrap()), 2.718);
  }
}

use std::{collections::HashMap, fmt::Display, io::Write};
use nom::{IResult, bytes::complete::{tag, take_while1}};

mod php_error;
use php_error::PhpError;

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

struct Parser;
impl Parser {
  fn prefix(src: &str) -> IResult<&str, &str, PhpError<&str>> {
    let (src, value) = take_while1(|s: char| s.is_ascii_alphanumeric() || s == '_')(src)?;
    let (src, _) = tag(":")(src)?;
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
  fn str_val(src: &str) -> IResult<&str, String, PhpError<&str>> {
    let (src, _) = tag("\"")(src)?;
    let mut out = String::new();
    let mut size = 0;
    let mut slash = false;
    for c in src.chars() {
      if slash {
        out.push(c);
        slash = false;
      } else {
        match c {
          '\\' => slash = true,
          '"' => break,
          c => out.push(c),
        }
      }
      size += c.len_utf8();
    }
    let src = &src[size..];
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
}

pub trait LuaType: 'static + Display {
}
pub trait LuaNameType: LuaType + Eq + std::hash::Hash {}
impl LuaNameType for LuaString {}
impl LuaNameType for LuaInteger {}

#[derive(PartialEq, Eq, Hash, Default)]
pub struct LuaString(String);
impl LuaString {
  fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, _) = Parser::prefix("s")?;
    let (src, ch_len) = Parser::usize_val(src)?;
    let (src, val) = Parser::str_val(src)?;
    if val.len() != ch_len as usize {
      Err(PhpError::BadLength(ch_len as _, val.len() as _).into())
    } else {
      Ok((src, Self::from(val)))
    }
  }
}
impl<T: Into<String>> From<T> for LuaString {
  fn from(src: T) -> Self {
    let src: String = src.into();
    Self(src)
  }
}
impl Display for LuaString {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, r#""{}""#, self.0)
  }
}
impl LuaType for LuaString {}
//transparent_lua!(LuaString, String);
#[derive(PartialEq, Eq, Hash, Default)]
pub struct LuaInteger(i32);
impl Display for LuaInteger {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl LuaType for LuaInteger {}
transparent_lua!(LuaInteger, i32);
#[derive(Default)]
pub struct LuaFloat(f32);
impl Display for LuaFloat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0.is_nan() {
      let sign = if self.0.is_sign_negative() { "-" } else { "" };
      write!(f, "{}nan", sign)
    } else if self.0.is_infinite() {
      let sign = if self.0.is_sign_negative() { "-" } else { "" };
      write!(f, "{}inf", sign)
    } else {
      write!(f, "{}", self.0)
    }
  }
}
impl LuaType for LuaFloat {}
transparent_lua!(LuaFloat, f32);
#[derive(Default)]
pub struct LuaTable<T: LuaNameType>(HashMap<T, Box<dyn LuaType>>);
impl<T: LuaNameType> AsMut<HashMap<T, Box<dyn LuaType>>> for LuaTable<T> {
  fn as_mut(&mut self) -> &mut HashMap<T, Box<dyn LuaType>> {
    &mut self.0
  }
}
impl<T: LuaNameType> AsRef<HashMap<T, Box<dyn LuaType>>> for LuaTable<T> {
  fn as_ref(&self) -> &HashMap<T, Box<dyn LuaType>> {
    &self.0
  }
}
impl<T: LuaNameType> LuaTable<T> {
  pub fn insert<PB: LuaType, A: Into<T>, B: Into<PB>>(&mut self, property: A, value: B) {
    self.as_mut().insert(property.into(), Box::new(value.into()));
  }
  pub fn insert_string<A: Into<T>, B: Into<LuaString>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_integer<A: Into<T>, B: Into<LuaInteger>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_float<A: Into<T>, B: Into<LuaFloat>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_string_table<A: Into<T>, B: Into<LuaTable<LuaString>>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_integer_table<A: Into<T>, B: Into<LuaTable<LuaInteger>>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
}
impl<T: LuaNameType> Display for LuaTable<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("{")?;
    let mut i = self.0.iter().peekable();
    while let Some((name, val)) = i.next() {
      write!(f, "[{}]={}", name, val)?;
      if i.peek().is_some() { f.write_str(",")?; }
    }
    f.write_str("}")
  }
}
impl<T: LuaNameType> LuaType for LuaTable<T> {}
#[derive(Default)]
pub struct LuaArray<T: LuaNameType, V: LuaType>(HashMap<T, V>);
impl<T: LuaNameType, V: LuaType> AsMut<HashMap<T, V>> for LuaArray<T, V> {
  fn as_mut(&mut self) -> &mut HashMap<T, V> {
    &mut self.0
  }
}
impl<T: LuaNameType, V: LuaType> AsRef<HashMap<T, V>> for LuaArray<T, V> {
  fn as_ref(&self) -> &HashMap<T, V> {
    &self.0
  }
}
impl<T: LuaNameType, V: LuaType> Display for LuaArray<T, V> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("{")?;
    let mut i = self.0.iter().peekable();
    while let Some((name, val)) = i.next() {
      write!(f, "[{}]={}", name, val)?;
      if i.peek().is_some() { f.write_str(",")?; }
    }
    f.write_str("}")
  }
}
impl<T: LuaNameType, V: LuaType> LuaType for LuaArray<T, V> {}
impl<T: LuaNameType, V: LuaType> From<LuaArray<T, V>> for LuaTable<T> {
  fn from(src: LuaArray<T, V>) -> Self {
    let mut out: HashMap<T, Box<dyn LuaType>> = HashMap::with_capacity(src.0.len());
    for (a, b) in src.0 {
      out.insert(a, Box::new(b));
    }
    Self(out)
  }
}

pub enum ToLuaMessage {
  LoadString { text: LuaString, name: LuaString },
  Call { id: LuaInteger, args: LuaTable<LuaString> },
  RegisterLibrary { name: LuaString, functions: LuaArray<LuaString, LuaString> },
  GetStatus,
  CleanupChunks { ids: LuaArray<LuaInteger, LuaInteger> },
  Quit,
  Testquit,
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
    }
    t
  }
}

pub enum FromLuaMessage {
  Call { id: LuaInteger, args: LuaTable<LuaString> },
}

pub enum LuaResponse {
  Return { values: LuaTable<LuaString> },
  Error { value: LuaString, trace: LuaTable<LuaString> },
}

pub use lua_array::LuaArray;
use lua_bool::LuaBool;
pub use lua_float::LuaFloat;
pub use lua_integer::LuaInteger;
use lua_null::LuaNull;
pub use lua_string::LuaString;
pub use lua_table::LuaTable;
use nom::{IResult, bytes::complete::{tag, take_while1}};
use std::{any::Any, fmt::Display, io::Write};

mod php_error;
mod lua_string;
mod lua_integer;
mod lua_float;
mod lua_table;
mod lua_array;
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
  fn any_lua(src: &str) -> IResult<&str, Box<dyn LuaType>, PhpError<&str>> {
    LuaString::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) })
      .or_else(|_| LuaBool::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
      .or_else(|_| LuaNull::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
      .or_else(|_| LuaInteger::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
      .or_else(|_| LuaFloat::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
      .or_else(|_| LuaTable::<LuaInteger>::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
      .or_else(|_| LuaTable::<LuaString>::parse(src).map(|(a, b)| -> (&str, Box<dyn LuaType>) { (a, Box::new(b)) }))
  }
}

pub trait LuaType: 'static + Display + std::fmt::Debug + Any {
  fn as_any(&self) -> &dyn Any;
}
pub trait LuaNameType: LuaType + Eq + std::hash::Hash {
  fn try_from_string(src: LuaString) -> Result<Box<Self>, LuaString>;
  fn try_from_integer(src: LuaInteger) -> Result<Box<Self>, LuaInteger>;
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
    let field = val.as_ref().get(&0.into()).unwrap();
    assert_eq!(bool::from(lua_as_x::<LuaBool>(field.as_ref()).unwrap()), true);
    let field = val.as_ref().get(&1.into()).unwrap();
    assert!(lua_as_x::<LuaNull>(field.as_ref()).is_some());
    let field = val.as_ref().get(&2.into()).unwrap();
    assert_eq!(f32::from(lua_as_x::<LuaFloat>(field.as_ref()).unwrap()), -421000000.0);
    let field = val.as_ref().get(&3.into()).unwrap();
    assert_eq!(lua_as_x::<LuaString>(field.as_ref()).unwrap().as_ref(), "A to Z");
  }
  let (last, val): (_, LuaTable<LuaString>) = LuaTable::parse(r#"a:2:{i:42;b:1;s:6:"A to Z";a:3:{i:0;i:1;i:1;i:2;i:2;i:3;}}"#).unwrap();
  assert!(last.is_empty());
  assert!(val.object.is_none());
  {
    let field = val.as_ref().get(&"42".into()).unwrap();
    assert_eq!(bool::from(lua_as_x::<LuaBool>(field.as_ref()).unwrap()), true);
    let field = val.as_ref().get(&"A to Z".into()).unwrap();
    let val = lua_as_x::<LuaTable<LuaInteger>>(field.as_ref()).unwrap();

    for i in 0..=2 {
      let field = val.as_ref().get(&i.into()).unwrap();
      assert_eq!(i32::from(lua_as_x::<LuaInteger>(field.as_ref()).unwrap()), i+1);
    }
  }
  let (last, val): (_, LuaTable<LuaString>) = LuaTable::parse(r#"O:8:"stdClass":2:{s:4:"John";d:3.14;s:4:"Jane";d:2.718;}"#).unwrap();
  assert!(last.is_empty());
  assert!(val.object.as_ref().map(|v| String::from(v.clone())) == Some("stdClass".to_owned()));
  {
    let field = val.as_ref().get(&"John".into()).unwrap();
    assert_eq!(f32::from(lua_as_x::<LuaFloat>(field.as_ref()).unwrap()), 3.14);
    let field = val.as_ref().get(&"Jane".into()).unwrap();
    assert_eq!(f32::from(lua_as_x::<LuaFloat>(field.as_ref()).unwrap()), 2.718);
  }
}

fn lua_as_x<S: Any + LuaType>(src: &dyn LuaType) -> Option<&S> {
  (*src.as_any()).downcast_ref::<S>()
}

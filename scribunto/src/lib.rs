use std::{collections::HashMap, fmt::Display, io::Write, str::FromStr};

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

pub trait LuaType: 'static + Display + FromStr {}
pub trait LuaNameType: LuaType + Eq + std::hash::Hash {}
impl LuaNameType for LuaString {}
impl LuaNameType for LuaInteger {}

#[derive(PartialEq, Eq, Hash, Default)]
pub struct LuaString(String);
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
    write!(f, "{}", self.0)
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

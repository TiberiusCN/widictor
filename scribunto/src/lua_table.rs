use std::{collections::HashMap, fmt::Display};
use nom::IResult;

use crate::{LuaFloat, LuaInteger, LuaNameType, LuaString, LuaType, Parser, lua_bool::LuaBool, lua_null::LuaNull, php_error::PhpError};

macro_rules! any {
	($raw: ty, $kind: expr, $pkind: path, $asraw: ident) => {
    impl From<$raw> for AnyLua {
      fn from(me: $raw) -> Self {
        $kind(me)
      }
    }
    impl AnyLua {
      fn $asraw(&self) -> Option<&$raw> {
        match &self {
          $pkind(me) => Some(me),
          _ => None,
        }
      }
    }
	};
}

#[derive(Debug, Clone)]
pub enum AnyLua {
  String(LuaString),
  Float(LuaFloat),
  Null(LuaNull),
  Bool(LuaBool),
  Integer(LuaInteger),
  StringTable(LuaTable<LuaString>),
  IntegerTable(LuaTable<LuaInteger>),
}
impl std::fmt::Display for AnyLua {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AnyLua::String(v) => v.fmt(f),
      AnyLua::Float(v) => v.fmt(f),
      AnyLua::Null(v) => v.fmt(f),
      AnyLua::Bool(v) => v.fmt(f),
      AnyLua::Integer(v) => v.fmt(f),
      AnyLua::StringTable(v) => v.fmt(f),
      AnyLua::IntegerTable(v) => v.fmt(f),
    }
  }
}
any!(LuaString, AnyLua::String, AnyLua::String, as_string);
any!(LuaFloat, AnyLua::Float, AnyLua::Float, as_float);
any!(LuaNull, AnyLua::Null, AnyLua::Null, as_null);
any!(LuaBool, AnyLua::Bool, AnyLua::Bool, as_bool);
any!(LuaInteger, AnyLua::Integer, AnyLua::Integer, as_integer);
any!(LuaTable<LuaString>, AnyLua::StringTable, AnyLua::StringTable, as_string_table);
any!(LuaTable<LuaInteger>, AnyLua::IntegerTable, AnyLua::IntegerTable, as_integer_table);

#[derive(Default, Debug, Clone)]
pub struct LuaTable<T: LuaNameType> {
  pub value: HashMap<T, Box<AnyLua>>,
  pub object: Option<String>,
}
impl<T: LuaNameType> AsMut<HashMap<T, Box<AnyLua>>> for LuaTable<T> {
  fn as_mut(&mut self) -> &mut HashMap<T, Box<AnyLua>> {
    &mut self.value
  }
}
impl<T: LuaNameType> AsRef<HashMap<T, Box<AnyLua>>> for LuaTable<T> {
  fn as_ref(&self) -> &HashMap<T, Box<AnyLua>> {
    &self.value
  }
}
impl<T: LuaNameType> LuaTable<T> {
  pub fn len(&self) -> usize {
    self.value.len()
  }
  pub fn insert<A: Into<T>, B: Into<AnyLua>>(&mut self, property: A, value: B) {
    self.as_mut().insert(property.into(), Box::new(value.into()));
  }
  pub fn insert_string<A: Into<T>, B: Into<LuaString>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_bool<A: Into<T>, B: Into<LuaBool>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_null<A: Into<T>, B: Into<LuaNull>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_integer<A: Into<T>, B: Into<LuaInteger>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_float<A: Into<T>, B: Into<LuaFloat>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_string_table<A: Into<T>, B: Into<LuaTable<LuaString>>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  pub fn insert_integer_table<A: Into<T>, B: Into<LuaTable<LuaInteger>>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value.into());
  }
  fn get<A: Into<T>>(&self, property: A) -> Option<&AnyLua> {
    self.value.get(&property.into()).map(Box::as_ref)
  }
  pub fn get_string<A: Into<T>>(&self, property: A) -> Option<&LuaString> {
    self.get(property).and_then(AnyLua::as_string)
  }
  pub fn get_bool<A: Into<T>>(&self, property: A) -> Option<&LuaBool> {
    self.get(property).and_then(AnyLua::as_bool)
  }
  pub fn get_null<A: Into<T>>(&self, property: A) -> Option<&LuaNull> {
    self.get(property).and_then(AnyLua::as_null)
  }
  pub fn get_integer<A: Into<T>>(&self, property: A) -> Option<&LuaInteger> {
    self.get(property).and_then(AnyLua::as_integer)
  }
  pub fn get_float<A: Into<T>>(&self, property: A) -> Option<&LuaFloat> {
    self.get(property).and_then(AnyLua::as_float)
  }
  pub fn get_string_table<A: Into<T>>(&self, property: A) -> Option<&LuaTable<LuaString>> {
    self.get(property).and_then(AnyLua::as_string_table)
  }
  pub fn get_integer_table<A: Into<T>>(&self, property: A) -> Option<&LuaTable<LuaInteger>> {
    self.get(property).and_then(AnyLua::as_integer_table)
  }
}
impl<T: LuaNameType> Display for LuaTable<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("{")?;
    let mut i = self.value.iter().peekable();
    while let Some((name, val)) = i.next() {
      write!(f, "[{}]={}", name, val)?;
      if i.peek().is_some() { f.write_str(",")?; }
    }
    f.write_str("}")
  }
}
impl<T: LuaNameType> LuaType for LuaTable<T> {}
impl<T: LuaNameType> LuaTable<T> {
  pub fn into_iter(self) -> impl Iterator<Item = (T, Box<AnyLua>)> {
    self.value.into_iter()
  }
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (mut src, prefix) = Parser::prefix(src)?;
    let object = match prefix {
      "a" => None,
      "O" => {
        let (tmp, ch_len) = Parser::usize_val(src)?;
        let (tmp, _) = Parser::separator(tmp)?;
        let (tmp, val) = Parser::str_val(tmp, ch_len)?;
        let (tmp, _) = Parser::separator(tmp)?;
        src = tmp;
        Some(val)
      },
      s => return Err(PhpError::UnexpectedPrefix("a/O", s.to_string()).into()),
    };
    let (src, pairs) = Parser::usize_val(src)?;
    let (src, _) = Parser::separator(src)?;
    let (src, _) = Parser::open(src)?;
    let mut fields = HashMap::new();
    let mut src = src;
    for _ in 0..pairs {
      if let Ok((tmp, field)) = LuaString::parse(src) {
        let (tmp, any) = Parser::any_lua(tmp)?;
        fields.insert(*T::try_from_string(field).map_err(|_| PhpError::BadType)?, Box::new(any));
        src = tmp;
      } else {
        let (tmp, field) = LuaInteger::parse(src)?;
        let (tmp, any) = Parser::any_lua(tmp)?;
        fields.insert(*T::try_from_integer(field).map_err(|_| PhpError::BadType)?, Box::new(any));
        src = tmp;
      }
    }
    let (src, _) = Parser::close(src)?;
    Ok((src, Self {
      value: fields,
      object,
    }))
  }
}

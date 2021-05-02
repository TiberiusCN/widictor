use std::{collections::HashMap, fmt::Display};
use nom::IResult;

use crate::{LuaArray, LuaFloat, LuaInteger, LuaNameType, LuaString, LuaType, Parser, lua_bool::LuaBool, lua_null::LuaNull, php_error::PhpError};

#[derive(Default, Debug)]
pub struct LuaTable<T: LuaNameType> {
  value: HashMap<T, Box<dyn LuaType>>,
  pub object: Option<String>,
}
impl<T: LuaNameType> AsMut<HashMap<T, Box<dyn LuaType>>> for LuaTable<T> {
  fn as_mut(&mut self) -> &mut HashMap<T, Box<dyn LuaType>> {
    &mut self.value
  }
}
impl<T: LuaNameType> AsRef<HashMap<T, Box<dyn LuaType>>> for LuaTable<T> {
  fn as_ref(&self) -> &HashMap<T, Box<dyn LuaType>> {
    &self.value
  }
}
impl<T: LuaNameType> LuaTable<T> {
  pub fn len(&self) -> usize {
    self.value.len()
  }
  pub fn insert<PB: LuaType, A: Into<T>, B: Into<PB>>(&mut self, property: A, value: B) {
    self.as_mut().insert(property.into(), Box::new(value.into()));
  }
  pub fn insert_string<A: Into<T>, B: Into<LuaString>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_bool<A: Into<T>, B: Into<LuaBool>>(&mut self, property: A, value: B) {
    self.insert(property.into(), value);
  }
  pub fn insert_null<A: Into<T>, B: Into<LuaNull>>(&mut self, property: A, value: B) {
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
  pub fn get<PB: LuaType, A: Into<T>, B: LuaType>(&self, property: A) -> Option<&B> {
    let field = self.value.get(&property.into())?;
    lua_as_x::<B>(field.as_ref())
  }
  pub fn get_string<A: Into<T>>(&self, property: A) -> Option<&LuaString> {
    self.get::<LuaString, _, _>(property)
  }
  pub fn get_bool<A: Into<T>>(&self, property: A) -> Option<&LuaBool> {
    self.get::<LuaBool, _, _>(property)
  }
  pub fn get_null<A: Into<T>>(&self, property: A) -> Option<&LuaNull> {
    self.get::<LuaNull, _, _>(property)
  }
  pub fn get_integer<A: Into<T>>(&self, property: A) -> Option<&LuaInteger> {
    self.get::<LuaInteger, _, _>(property)
  }
  pub fn get_float<A: Into<T>>(&self, property: A) -> Option<&LuaFloat> {
    self.get::<LuaFloat, _, _>(property)
  }
  pub fn get_string_table<A: Into<T>>(&self, property: A) -> Option<&LuaTable<LuaString>> {
    self.get::<LuaTable<LuaString>, _, _>(property)
  }
  pub fn get_integer_table<A: Into<T>>(&self, property: A) -> Option<&LuaTable<LuaInteger>> {
    self.get::<LuaTable<LuaInteger>, _, _>(property)
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
impl<T: LuaNameType, V: LuaType> From<LuaArray<T, V>> for LuaTable<T> {
  fn from(src: LuaArray<T, V>) -> Self {
    let mut out: HashMap<T, Box<dyn LuaType>> = HashMap::with_capacity(src.as_ref().len());
    for (a, b) in src.into_iter() {
      out.insert(a, Box::new(b));
    }
    Self {
      value: out,
      object: None,
    }
  }
}
impl<T: LuaNameType> LuaType for LuaTable<T> {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}
impl<T: LuaNameType> LuaTable<T> {
  pub fn into_iter(self) -> impl Iterator<Item = (T, Box<dyn LuaType>)> {
    self.value.into_iter()
  }
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (mut src, prefix) = Parser::prefix(src)?;
    let object = match prefix {
      "a" => None,
      "O" => {
        let (tmp, ch_len) = Parser::usize_val(src)?;
        let (tmp, _) = Parser::separator(tmp)?;
        let (tmp, val) = Parser::str_val(tmp)?;
        let (tmp, _) = Parser::separator(tmp)?;
        if val.len() != ch_len as usize {
          return Err(PhpError::BadLength(ch_len as _, val.len() as _).into());
        }
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
        fields.insert(*T::try_from_string(field).map_err(|_| PhpError::BadType)?, any);
        src = tmp;
      } else {
        let (tmp, field) = LuaInteger::parse(src)?;
        let (tmp, any) = Parser::any_lua(tmp)?;
        fields.insert(*T::try_from_integer(field).map_err(|_| PhpError::BadType)?, any);
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

fn lua_as_x<S: std::any::Any + LuaType>(src: &dyn LuaType) -> Option<&S> {
  (*src.as_any()).downcast_ref::<S>()
}

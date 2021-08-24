#[allow(unused)]
use crate::scribunto as m;
use m::{lua_chunk::LuaChunk, php_error::PhpError, LuaNameType, LuaString, LuaType, Parser};

use nom::IResult;
use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Default, Debug, Clone)]
pub struct LuaInteger(i32);
impl LuaInteger {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, prefix) = Parser::prefix(src)?;
    if prefix != "i" {
      return Err(PhpError::UnexpectedPrefix("i", prefix.to_string()).into());
    }
    let (src, val) = Parser::i32_val(src)?;
    let (src, _) = Parser::finite(src)?;
    Ok((src, Self::from(val)))
  }
  pub fn to_raw(self) -> i32 {
    self.0
  }
  pub fn as_raw(&self) -> &i32 {
    &self.0
  }
  pub fn to_chunk(self) -> LuaChunk {
    LuaChunk::new(&self)
  }
}
impl Display for LuaInteger {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl LuaType for LuaInteger {}
crate::transparent_lua!(LuaInteger, i32);
impl LuaNameType for LuaInteger {
  fn try_from_string(src: LuaString) -> Result<Box<Self>, LuaString> {
    let s: String = src.to_raw();
    if let Ok(i) = s.parse::<i32>() {
      Ok(Box::new(Self::from(i)))
    } else {
      Err(s.into())
    }
  }
  fn try_from_integer(src: LuaInteger) -> Result<Box<Self>, LuaInteger> {
    Ok(Box::new(src))
  }
}

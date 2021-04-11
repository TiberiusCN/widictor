use std::fmt::Display;
use nom::IResult;
use crate::{LuaNameType, LuaString, LuaType, Parser, php_error::PhpError};

#[derive(PartialEq, Eq, Hash, Default, Debug)]
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
  pub fn to_taw(self) -> i32 { self.0 }
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

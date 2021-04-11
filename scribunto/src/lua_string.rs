use std::fmt::Display;
use nom::IResult;
use crate::{LuaInteger, LuaNameType, LuaType, Parser, php_error::PhpError};

#[derive(PartialEq, Eq, Hash, Default, Debug)]
pub struct LuaString(String);
impl LuaString {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, prefix) = Parser::prefix(src)?;
    if prefix != "s" {
      return Err(PhpError::UnexpectedPrefix("s", prefix.to_string()).into());
    }
    let (src, ch_len) = Parser::usize_val(src)?;
    let (src, _) = Parser::separator(src)?;
    let (src, val) = Parser::str_val(src)?;
    let (src, _) = Parser::finite(src)?;
    if val.len() != ch_len as usize {
      Err(PhpError::BadLength(ch_len as _, val.len() as _).into())
    } else {
      Ok((src, Self::from(val)))
    }
  }
  pub fn to_raw(self) -> String { self.0 }
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
impl LuaNameType for LuaString {
  fn try_from_string(src: LuaString) -> Result<Box<Self>, LuaString> {
    Ok(Box::new(src))
  }
  fn try_from_integer(src: LuaInteger) -> Result<Box<Self>, LuaInteger> {
    Ok(Box::new(Self::from(format!("{}", i32::from(src)))))
  }
}

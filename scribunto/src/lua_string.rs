use std::fmt::Display;
use nom::IResult;
use crate::{LuaInteger, LuaNameType, LuaType, Parser, php_error::PhpError};

#[derive(PartialEq, Eq, Hash, Default, Debug, Clone)]
pub struct LuaString(String);
impl LuaString {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, prefix) = Parser::prefix(src)?;
    if prefix != "s" {
      return Err(PhpError::UnexpectedPrefix("s", prefix.to_string()).into());
    }
    let (src, ch_len) = Parser::usize_val(src)?;
    let (src, _) = Parser::separator(src)?;
    let (src, val) = Parser::str_val(src, ch_len)?;
    let (src, _) = Parser::finite(src)?;
    Ok((src, Self::from(val)))
  }
  pub fn to_raw(self) -> String { self.0 }
  pub fn as_raw(&self) -> &str { self.0.as_str() }
}
impl<T: Into<String>> From<T> for LuaString {
  fn from(src: T) -> Self {
    let src: String = src.into();
    Self(src)
  }
}
impl AsRef<str> for LuaString {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
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

#[allow(unused)]
use crate::scribunto as m;
use m::{php_error::PhpError, LuaType, Parser};

use nom::IResult;
use std::fmt::Display;

#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct LuaBool(bool);
impl LuaBool {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, prefix) = Parser::prefix(src)?;
    if prefix != "b" {
      return Err(PhpError::UnexpectedPrefix("b", prefix.to_string()).into());
    }
    let (src, val) = Parser::i32_val(src)?;
    let (src, _) = Parser::finite(src)?;
    Ok((src, Self::from(val != 0)))
  }
  pub fn to_raw(self) -> bool {
    self.0
  }
  pub fn as_raw(&self) -> &bool {
    &self.0
  }
}
impl Display for LuaBool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl LuaType for LuaBool {}
crate::transparent_lua!(LuaBool, bool);

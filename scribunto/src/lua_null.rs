use std::fmt::Display;
use nom::IResult;
use crate::{LuaType, Parser, php_error::PhpError};

#[derive(Default, Debug)]
pub struct LuaNull(());
impl LuaNull {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, _) = Parser::null_val(src)?;
    let (src, _) = Parser::finite(src)?;
    Ok((src, Self(())))
  }
  pub fn to_raw(self) -> () { self.0 }
  pub fn as_raw(&self) -> &() { &self.0 }
}
impl Display for LuaNull {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("null")
  }
}
impl LuaType for LuaNull {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}
crate::transparent_lua!(LuaNull, ());

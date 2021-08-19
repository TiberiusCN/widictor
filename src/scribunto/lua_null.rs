#[allow(unused)]
use crate::scribunto as m;
use m::{LuaType, Parser, php_error::PhpError};

use nom::IResult;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
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
impl LuaType for LuaNull {}
crate::transparent_lua!(LuaNull, ());

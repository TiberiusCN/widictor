#[allow(unused)]
use crate::scribunto as m;
use m::{php_error::PhpError, LuaType, Parser};

use nom::IResult;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
pub struct LuaFloat(f32);
impl LuaFloat {
  pub fn parse(src: &str) -> IResult<&str, Self, PhpError<&str>> {
    let (src, prefix) = Parser::prefix(src)?;
    if prefix != "d" {
      return Err(PhpError::UnexpectedPrefix("d", prefix.to_string()).into());
    }
    let (src, val) = Parser::f32_val(src)?;
    let (src, _) = Parser::finite(src)?;
    Ok((src, Self::from(val)))
  }
  pub fn to_raw(self) -> f32 {
    self.0
  }
  pub fn as_raw(&self) -> &f32 {
    &self.0
  }
}
impl Display for LuaFloat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0.is_nan() {
      let sign = if self.0.is_sign_negative() { "-" } else { "" };
      write!(f, "{}nan", sign)
    } else if self.0.is_infinite() {
      let sign = if self.0.is_sign_negative() { "-" } else { "" };
      write!(f, "{}inf", sign)
    } else {
      write!(f, "{}", self.0)
    }
  }
}
impl LuaType for LuaFloat {}
crate::transparent_lua!(LuaFloat, f32);

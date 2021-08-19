#[allow(unused)]
use crate::scribunto as m;
use m::{LuaInteger, LuaType};

use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Default, Debug, Clone)]
pub struct LuaChunk(i32);
impl LuaChunk {
  pub fn new(src: &LuaInteger) -> Self { Self(*src.as_raw()) }
  pub fn to_raw(self) -> i32 { self.0 }
  pub fn as_raw(&self) -> &i32 { &self.0 }
  pub fn to_integer(self) -> LuaInteger { LuaInteger::from(self.0) }
}
impl Display for LuaChunk {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "chunks[{}]", self.0)
  }
}
impl LuaType for LuaChunk {}
crate::transparent_lua!(LuaChunk, i32);

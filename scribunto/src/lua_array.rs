use std::{collections::HashMap, fmt::Display};
use crate::{LuaNameType, LuaType};

#[derive(Default, Debug)]
pub struct LuaArray<T: LuaNameType, V: LuaType>(HashMap<T, V>);
impl<T: LuaNameType, V: LuaType> AsMut<HashMap<T, V>> for LuaArray<T, V> {
  fn as_mut(&mut self) -> &mut HashMap<T, V> {
    &mut self.0
  }
}
impl<T: LuaNameType, V: LuaType> AsRef<HashMap<T, V>> for LuaArray<T, V> {
  fn as_ref(&self) -> &HashMap<T, V> {
    &self.0
  }
}
impl<T: LuaNameType, V: LuaType> Display for LuaArray<T, V> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("{")?;
    let mut i = self.0.iter().peekable();
    while let Some((name, val)) = i.next() {
      write!(f, "[{}]={}", name, val)?;
      if i.peek().is_some() { f.write_str(",")?; }
    }
    f.write_str("}")
  }
}
impl<T: LuaNameType, V: LuaType> LuaType for LuaArray<T, V> {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}
impl<T: LuaNameType, V: LuaType> LuaArray<T, V> {
  pub fn into_iter(self) -> impl Iterator<Item = (T, V)> {
    self.0.into_iter()
  }
}

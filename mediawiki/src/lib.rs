use mlua::prelude::*;
use mlua::Function;
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  Lua(#[from] LuaError),
  #[error("Module {0} not found")]
  NoSuchModule(String),
}

pub struct MediaWiki {
  pub lua: &'static Lua,
  pub deps: Vec<String>,
  pub paths: Vec<PathBuf>,
}

impl MediaWiki {
  pub fn new() -> Result<Self, Error> {
    let paths: Vec<PathBuf> = std::env::var("WIDICTOR_MW_MODULES").expect("WIDICTOR_MW_MODULES not found").split(':').map(PathBuf::from).collect();
    let mut modules = String::new();
    for path in &paths {
      modules += &format!("{}/?.lua;", path.display().to_string());
    }
    let lua_path = std::env::var("LUA_PATH").unwrap_or_default();
    std::env::set_var("LUA_PATH", &format!("{}{}", modules, lua_path));

    let lua = Lua::new();
    let lua = lua.into_static();

    Ok(Self {
      lua,
      deps: Vec::new(),
      paths,
    })
  }

  pub fn init(&mut self) -> Result<(), Error> {
    let lua = self.lua.clone();

    lua.set_hook(mlua::HookTriggers {
      every_line: true, ..Default::default()
    }, |_lua, debug| {
      println!("line {}: {}", debug.curr_line(), std::str::from_utf8(debug.source().short_src.unwrap()).unwrap());
      Ok(())
    }).unwrap();

    /*
    let mw = lua.create_table()?;

    {
      let deps = self.deps.clone();
      let require: Function = lua.clone().globals().get("require")?;
      let require = lua.create_function(move |_: &Lua, module: String| -> LuaResult<LuaTable> {
        let module = module.as_str().strip_prefix("Module:").unwrap_or(&module);
        if deps.iter().position(|v| v == module).is_none() {
          Err(LuaError::RuntimeError(format!("dependency {} not found", module)))
        } else {
          require.call(module.to_owned())
        }
      })?;
      lua.globals().set("require", require.clone())?;
      mw.set("loadData", require)?;
    }

    let unicode = lua.create_table()?;
    unicode.set("char", lua.create_function(mw::mw_unicode_char)?)?;
    mw.set("ustring", unicode)?;
    // https://www.mediawiki.org/wiki/Extension:Scribunto/Lua_reference_manual#Ustring_library

    lua.globals().set("mw", mw)?;
    */

    Ok(())
  }

  pub fn check_dependency(&mut self, dep: &str) -> Result<(), Error> {
    let mut target = false;
    if self.deps.iter().position(|v| v == dep).is_some() { return Ok(()); }

    for dir_path in &self.paths {
      let path = dir_path.join(format!("{}.lua", dep));
      if path.exists() {
        target = true;
        break;
      }
    }
    if !target {
      for dir_path in &self.paths {
        let path = dir_path.join(format!("{}.lua", dep));
        let mut dir = path.clone();
        dir.pop();
        let _ = std::fs::create_dir_all(dir);
        if let Ok(mut module) = std::fs::File::create(&path) {
          use std::io::Write;
          let data = get(&format!("Module:{}", dep));
          module.write_all(data.as_bytes()).unwrap();
          target = true;

          break;
        }
      }

      if !target {
        return Err(Error::NoSuchModule(dep.to_owned().to_string()));
      }
    }

    self.deps.push(dep.to_owned());

    Ok(())
  }

  pub fn execute(&self, module: &str, function: &str) -> Result<String, LuaError> {
    let out = self.lua.load(&format!(r#"
local module = require("Module:{}")
module.{}()
      "#, module, function))
      .exec()?;
    panic!("BE: {:#?}", out)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test() {
    let mw = MediaWiki::new().unwrap();
    mw.lua.load("mw.print(\"success\")").exec().unwrap();
  }
}

pub fn trace(error: &dyn std::error::Error) -> ! {
  println!("{}", error);
  if let Some(src) = error.source() {
    trace(src)
  } else {
    panic!()
  }
}

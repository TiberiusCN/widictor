use mlua::prelude::*;
use mlua::Function;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  Lua(#[from] LuaError),
  #[error("Module {0} not found")]
  NoSuchModule(String),
}

pub struct MediaWiki {
  pub lua: &'static Lua,
}

impl MediaWiki {
  pub fn new() -> Result<Self, Error> {
    let lua = Lua::new();
    let lua = lua.into_static();
    lua.set_hook(mlua::HookTriggers {
      every_line: true, ..Default::default()
    }, |_lua, debug| {
      println!("line {}: {}", debug.curr_line(), std::str::from_utf8(debug.source().short_src.unwrap()).unwrap());
      Ok(())
    }).unwrap();

    let mw = lua.create_table()?;

    {
      let require: Function = lua.clone().globals().get("require")?;
      let require = lua.create_function(move |_: &Lua, module: String| -> LuaResult<LuaTable> {
        let module = module.as_str().strip_prefix("Module:").unwrap_or(&module);
        println!("REQ: {}", module);
        require.call(module.to_owned())
      })?;
      lua.globals().set("require", require.clone())?;
      mw.set("loadData", require)?;
    }

    let unicode = lua.create_table()?;
    mw.set("ustring", unicode)?;
    // https://www.mediawiki.org/wiki/Extension:Scribunto/Lua_reference_manual#Ustring_library

    lua.globals().set("mw", mw)?;

    Ok(Self {
      lua,
    })
  }

  pub fn check_dependencies(deps: &[&str]) -> Result<(), Error> {
    use std::path::PathBuf;

    let paths: Vec<PathBuf> = std::env::var("WIDICTOR_MW_MODULES").expect("WIDICTOR_MW_MODULES not found").split(':').map(PathBuf::from).collect();
    for dep in deps {
      let mut target = false;

      if !target {
        for dir_path in &paths {
          let path = dir_path.join(format!("{}.lua", dep));
          if path.exists() {
            target = true;
            break;
          }
        }
        if !target {
          for dir_path in &paths {
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
      }
    }

    let mut modules = String::new();
    for path in &paths {
      modules += &format!("{}/?.lua;", path.display().to_string());
    }
    let lua_path = std::env::var("LUA_PATH").unwrap_or_default();
    std::env::set_var("LUA_PATH", &format!("{}{}", modules, lua_path));
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

#[allow(unused)]
mod mw {
  use mlua::prelude::*;
  
  pub fn mw_proto(_: &Lua, value: String) -> LuaResult<()> {
    Ok(())
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

use serde_derive::*;
use std::collections::HashMap;
  
#[derive(Deserialize)]
struct ApiAnswer {
  query: ApiQuery,
}

#[derive(Deserialize)]
struct ApiQuery {
  pages: HashMap<String, ApiPage>,
}

#[derive(Deserialize)]
struct ApiPage {
  //pageid: u32,
  //ns: u32,
  //title: String,
  revisions: Vec<ApiRevision>,
}

#[derive(Deserialize)]
struct ApiRevision {
  //contentformat: String,
  //contentmodel: String,
  #[serde(rename = "*")]
  data: String,
}

pub fn get(page: &str) -> String {
  let resp = reqwest::blocking::get(&format!("https://en.wiktionary.org/w/api.php?action=query&prop=revisions&rvprop=content&format=json&titles={}", page)).unwrap();
  let resp: ApiAnswer = serde_json::from_reader(resp.bytes().unwrap().as_ref()).unwrap();
  resp.query.pages.iter().last().unwrap().1.revisions[0].data.clone()
}

pub fn trace(error: &dyn std::error::Error) -> ! {
  println!("{}", error);
  if let Some(src) = error.source() {
    trace(src)
  } else {
    panic!()
  }
}

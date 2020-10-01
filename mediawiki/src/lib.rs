use mlua::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("{0}")]
  Lua(#[from] LuaError),
  #[error("Module {0} not found")]
  NoSuchModule(String),
}

pub struct MediaWiki {
  pub lua: Lua,
}

impl MediaWiki {
  pub fn new() -> Result<Self, Error> {
    let lua = Lua::new();

    let mw = lua.create_table()?;

    //let require = globals.get("require")?;
    // require.cal::<_, ()>()?;
    mw.set("print", lua.create_function(mw::mw_print)?)?;

    lua.globals().set("mw", mw)?;

    Ok(Self {
      lua,
    })
  }

  pub fn check_dependencies(deps: &[&str]) -> Result<(), Error> {
    use std::path::PathBuf;

    let paths: Vec<PathBuf> = std::env::var("LUA_PATH").expect("LUA_PATH not found").split(':').map(PathBuf::from).collect();
    for dep in deps {
      let dep = format!("Module:{}", dep);
      let mut found = false;

      for stat in &["params"] {
        if stat == &dep {
          found = true;
          break;
        }
      };

      if !found {
        for path in &paths {
          let path = path.join(format!("{}.lua", dep));
          if path.exists() {
            found = true;
            break;
          }
        }
        if !found {
          for path in &paths {
            let path = path.join(format!("{}.lua", dep));
            if let Ok(mut module) = std::fs::File::create(path) {
              use std::io::Write;
              let data = get(&dep);
              module.write_all(data.as_bytes()).unwrap();

              found = true;
              break;
            }
          }
        }

        if !found { return Err(Error::NoSuchModule(dep.to_string())); }
      }
    }
    Ok(())
  }

  pub fn execute(&self, module: &str, function: &str) -> Result<String, LuaError> {
    let out = self.lua.load(&format!(r#"
local module = require("Module:{}")
require.{}()
      "#, module, function))
      .exec()?;
    panic!("{:#?}", out)
  }
}

#[allow(unused)]
mod mw {
  use mlua::prelude::*;
  
  pub fn mw_print(_: &Lua, value: String) -> LuaResult<()> {
    println!("{}", value);
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

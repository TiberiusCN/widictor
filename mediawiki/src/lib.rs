use mlua::prelude::*;

pub struct MediaWiki {
  pub lua: Lua,
}

impl MediaWiki {
  pub fn new() -> Result<Self, LuaError> {
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

  pub fn check_dependencies(deps: &[&str]) {
    use curl::easy::Easy;
    use std::io::Write;
    use std::path::PathBuf;

    let paths: Vec<PathBuf> = std::env::var("LUA_PATH").unwrap().split(':').map(PathBuf::from).collect();
    for dep in deps {
      let mut found = false;

      for stat in &["params"] {
        if stat == dep {
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
              let mut easy = Easy::new();
              easy.url(&format!("https://en.wiktionary.org/wiki/{}", dep)).unwrap();
              easy.write_function(move |data| {
                module.write_all(data).unwrap();
                Ok(data.len())
              }).unwrap();
              easy.perform().unwrap();

              found = true;
              break;
            }
          }

          if !found { panic!("Module {} not found", dep); }
        }
      }
    }
  }

  pub fn execute(&self, module: &str, function: &str) -> Result<String, LuaError> {
    let out = self.lua.load(&format!(r#"
local module = require {}
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

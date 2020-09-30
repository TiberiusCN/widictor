use mlua::*;

pub struct MediaWiki {
  pub lua: Lua,
}

impl MediaWiki {
  fn new() -> Self {
    let lua = Lua::new();

    let mw = lua.create_table()?;

    let require = globals.get("require")?;
    // require.cal::<_, ()>()?;
    let print = lua.create_function(|_, strings: Variadic<String>| {
      Ok(())
    })?;

    lua.globals().set("mw", mw)?;

    Self {
      lua,
    }
  }
}

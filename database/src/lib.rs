use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
  #[error("Value {0} ({1}) not found")]
  ValueNotFound(String, &'static str),
}

pub struct Word {
  word: i64,
  base: Base,
}

impl Word {
  pub fn value(&self) -> Result<String, Error> {
    self.base.word_value(self.word)
  }
  pub fn tags(&self) -> Result<Vec<String>, Error> {
    self.base.word_tags(self.word)
  }
  pub fn forms(&self) -> Result<HashMap<String, String>, Error> {
    self.base.word_forms(self.word)
  }
  pub fn properties(&self) -> Result<HashMap<String, String>, Error> {
    self.base.word_properties(self.word)
  }
  pub fn property(&self, property: &str) -> Result<String, Error> {
    self.base.word_property(self.word, property)
  }
  pub fn insert_tag(&mut self, tag: &str) -> Result<(), Error> {
    self.base.insert_tag(self.word, tag)
  }
  pub fn insert_derived(&mut self, word: &str) -> Result<(), Error> {
    self.base.insert_derived(self.word, word)
  }
  pub fn insert_produced(&mut self, word: &str) -> Result<(), Error> {
    self.base.insert_produced(self.word, word)
  }
  pub fn insert_form(&mut self, form: &str, value: &str) -> Result<(), Error> {
    self.base.insert_form(self.word, form, value)
  }
  pub fn insert_property(&mut self, property: &str, value: &str) -> Result<(), Error> {
    self.base.insert_property(self.word, property, value)
  }
  pub fn key(&self) -> i64 {
    self.word
  }
  pub fn derived(&self) -> Result<Vec<Word>, Error> {
    self.base.word_derived(self.word)
  }
  pub fn produced(&self) -> Result<Vec<Word>, Error> {
    self.base.word_produced(self.word)
  }
}

#[derive(Clone)]
pub struct Base {
  connection: Arc<RwLock<Connection>>,
}

impl Base {
  pub fn create(path: &std::path::Path) -> Result<Self, Error> {
    {
      std::fs::File::create(path)?;
    }
    let me = Self::open(path)?;
    {
      let connection = me.connection.write().unwrap();
      connection.execute(
        "CREATE TABLE words (
          id INTEGER PRIMARY KEY,
          word TEXT NOT NULL,
          value TEXT NOT NULL
        )",
        params![]
      )?;
      connection.execute(
        "CREATE TABLE tags (
          id INTEGER PRIMARY KEY,
          tag TEXT NOT NULL UNIQUE
        )",
        params![]
      )?;
      connection.execute(
        "CREATE TABLE word_tags (
          word INTEGER,
          tag INTEGER,
          PRIMARY KEY (word, tag),
          FOREIGN KEY (word)
            REFERENCES words (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION,
          FOREIGN KEY (tag)
            REFERENCES tags (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION
        )",
        params![]
      )?;
      // mutation
      connection.execute(
        "CREATE TABLE forms (
          id INTEGER PRIMARY KEY,
          form TEXT NOT NULL UNIQUE
        )",
        params![]
      )?;
      connection.execute(
        "CREATE TABLE word_forms (
          word INTEGER,
          form INTEGER,
          value TEXT NOT NULL,
          PRIMARY KEY (word, form),
          FOREIGN KEY (word)
            REFERENCES words (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION,
          FOREIGN KEY (form)
            REFERENCES forms (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION
        )",
        params![]
      )?;
      // properties
      connection.execute(
        "CREATE TABLE properties (
          id INTEGER PRIMARY KEY,
          property TEXT NOT NULL UNIQUE
        )",
        params![]
      )?;
      connection.execute(
        "CREATE TABLE word_properties (
          word INTEGER,
          property INTEGER,
          value TEXT NOT NULL,
          PRIMARY KEY (word, property),
          FOREIGN KEY (word)
            REFERENCES words (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION,
          FOREIGN KEY (property)
            REFERENCES properties (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION
        )",
        params![]
      )?;
      // links
      connection.execute(
        "CREATE TABLE word_links (
          word INTEGER,
          etymology INTEGER,
          PRIMARY KEY (word, etymology),
          FOREIGN KEY (word)
            REFERENCES words (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION,
          FOREIGN KEY (etymology)
            REFERENCES words (id)
              ON DELETE CASCADE
              ON UPDATE NO ACTION
        )",
        params![]
      )?;
    }
    Ok(me)
  }

  pub fn open(path: &std::path::Path) -> Result<Self, Error> {
    let connection = Arc::new(RwLock::new(Connection::open(path)?));
    Ok(Self {
      connection
    })
  }

  pub fn insert_word(&mut self, word: &str, value: &str) -> Result<Word, Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "INSERT INTO words (word, value) VALUES (?1, ?2)",
      params![
        word,
        value,
      ]
    )?;
    let word = connection.last_insert_rowid();
    Ok(Word {
      word,
      base: self.clone(),
    })
  }

  fn new_tag(&mut self, tag: &str) -> Result<i64, Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "INSERT INTO tags (tag) VALUES (?1)",
      params![
        tag,
      ]
    )?;
    Ok(connection.last_insert_rowid())
  }

  fn new_form(&mut self, form: &str) -> Result<i64, Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "INSERT INTO forms (form) VALUES (?1)",
      params![
        form,
      ]
    )?;
    Ok(connection.last_insert_rowid())
  }

  fn new_property(&mut self, property: &str) -> Result<i64, Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "INSERT INTO properties (property) VALUES (?1)",
      params![
        property,
      ]
    )?;
    Ok(connection.last_insert_rowid())
  }

  pub fn search_word(&self, word: &str) -> Result<Vec<Word>, Error> {
    let words = {
      let connection = self.connection.read().unwrap();
      let mut stmt = connection.prepare("SELECT id FROM words WHERE word = ?1")?;
      let iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
        row.get(0)
      })?;
      let words = iter.collect::<Result<Vec<_>, _>>()?;
      if words.is_empty() {
        return Err(Error::ValueNotFound(word.to_owned(), "word"));
      }
      words.into_iter().map(|word| Word { word, base: self.clone() }).collect()
    };
    Ok(words)
  }

  pub fn get_word(&self, word: i64) -> Word {
    Word {
      word,
      base: self.clone(),
    }
  }

  pub fn search_word_or_form(&self, word: &str) -> Result<Vec<Word>, Error> {
    match self.search_word(word) {
      Ok(s) => return Ok(s),
      Err(Error::ValueNotFound(..)) => {},
      Err(e) => return Err(e),
    };
    self.search_form_of_word(word)
  }

  pub fn search_form_of_word(&self, word: &str) -> Result<Vec<Word>, Error> {
    let words = {
      let connection = self.connection.read().unwrap();
      let mut stmt = connection.prepare(
        "SELECT id FROM words
        INNER JOIN word_forms ON word_forms.word = words.id
        WHERE word_forms.value = ?1")?;
      let iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
        row.get(0)
      })?;
      let words = iter.collect::<Result<Vec<_>, _>>()?;
      if words.is_empty() {
        return Err(Error::ValueNotFound(word.to_owned(), "word form"));
      }
      words.into_iter().map(|word| Word { word, base: self.clone() }).collect()
    };
    Ok(words)
  }

  fn search_tag(&self, tag: &str) -> Result<i64, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare("SELECT id FROM tags WHERE tag = ?1")?;
    let mut iter = stmt.query_map(params![tag], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(tag.to_owned(), "tag"))??)
  }

  fn search_form(&self, form: &str) -> Result<i64, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare("SELECT id FROM forms WHERE form = ?1")?;
    let mut iter = stmt.query_map(params![form], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(form.to_owned(), "form"))??)
  }

  fn search_property(&self, property: &str) -> Result<i64, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare("SELECT id FROM properties WHERE property = ?1")?;
    let mut iter = stmt.query_map(params![property], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(property.to_owned(), "property"))??)
  }

  fn insert_tag(&mut self, word: i64, tag: &str) -> Result<(), Error> {
    let tag = match self.search_tag(tag) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => self.new_tag(tag)?,
      Err(e) => return Err(e),
    };
    self.connection.write().unwrap().execute(
      "INSERT INTO word_tags (word, tag) VALUES (?1, ?2)",
      params![
        word,
        tag,
      ]
    )?;
    Ok(())
  }

  fn insert_form(&mut self, word: i64, form: &str, value: &str) -> Result<(), Error> {
    let form = match self.search_form(form) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => self.new_form(form)?,
      Err(e) => return Err(e),
    };
    self.connection.write().unwrap().execute(
      "INSERT INTO word_forms (word, form, value) VALUES (?1, ?2, ?3)",
      params![
        word,
        form,
        value,
      ]
    )?;
    Ok(())
  }

  fn insert_derived(&mut self, word: i64, derived: &str) -> Result<(), Error> {
    let parents = match self.search_word(derived) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => return Ok(()),
      Err(e) => return Err(e),
    };
    for parent in parents {
      self.insert_link(parent.word, word)?;
    }
    Ok(())
  }
  fn insert_produced(&mut self, word: i64, produced: &str) -> Result<(), Error> {
    let children = match self.search_word(produced) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => return Ok(()),
      Err(e) => return Err(e),
    };
    for child in children {
      self.insert_link(word, child.word)?;
    }
    Ok(())
  }
  fn insert_link(&mut self, parent: i64, child: i64) -> Result<(), Error> {
    self.connection.write().unwrap().execute(
      "INSERT INTO word_links (word, etymology) VALUES (?1, ?2)",
      params![
        child,
        parent,
      ]
    )?;
    Ok(())
  }

  fn insert_property(&mut self, word: i64, property: &str, value: &str) -> Result<(), Error> {
    let property = match self.search_property(property) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => self.new_property(property)?,
      Err(e) => return Err(e),
    };
    self.connection.write().unwrap().execute(
      "INSERT INTO word_properties (word, property, value) VALUES (?1, ?2, ?3)",
      params![
        word,
        property,
        value,
      ]
    )?;
    Ok(())
  }

  fn word_tags(&self, word: i64) -> Result<Vec<String>, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT tags.tag FROM tags
        INNER JOIN word_tags ON word_tags.tag = tags.id
        WHERE word_tags.word = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<String, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.collect::<Result<Vec<_>, _>>()?)
  }

  fn word_derived(&self, word: i64) -> Result<Vec<Word>, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT word_links.etymology FROM word_links
        WHERE word_links.word = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.map(|v: Result<i64, rusqlite::Error>| -> Result<Word, Error> { Ok(Word { word: v?, base: self.clone() })}).collect::<Result<Vec<_>, _>>()?)
  }
  fn word_produced(&self, word: i64) -> Result<Vec<Word>, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT word_links.word FROM word_links
        WHERE word_links.etymology = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.map(|v: Result<i64, rusqlite::Error>| -> Result<Word, Error> { Ok(Word { word: v?, base: self.clone() })}).collect::<Result<Vec<_>, _>>()?)
  }

  fn word_forms(&self, word: i64) -> Result<HashMap<String, String>, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT forms.form, word_forms.value FROM forms
        INNER JOIN word_forms ON word_forms.form = forms.id
        WHERE word_forms.word = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<(String, String), rusqlite::Error> {
      Ok((row.get(0)?, row.get(1)?))
    })?;
    Ok(iter.collect::<Result<HashMap<String, String>, _>>()?)
  }

  fn word_properties(&self, word: i64) -> Result<HashMap<String, String>, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT properties.property, word_properties.value FROM properties
        INNER JOIN word_properties ON word_properties.property = properties.id
        WHERE word_properties.word = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<(String, String), rusqlite::Error> {
      Ok((row.get(0)?, row.get(1)?))
    })?;
    Ok(iter.collect::<Result<HashMap<String, String>, _>>()?)
  }

  fn word_property(&self, word: i64, property: &str) -> Result<String, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare(
      "SELECT word_properties.value FROM properties
        INNER JOIN word_properties ON word_properties.property = properties.id
        WHERE (word_properties.word, properties.property) = (?1, ?2)")?;
    let mut iter = stmt.query_map(params![word, property], |row| -> Result<String, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(word.to_string(), "property"))??)
  }

  fn word_value(&self, word: i64) -> Result<String, Error> {
    let connection = self.connection.read().unwrap();
    let mut stmt = connection.prepare("SELECT value FROM words WHERE id = ?1")?;
    let mut iter = stmt.query_map(params![word], |row| -> Result<String, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(word.to_string(), "value"))??)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn table() {
    let mut base = Base::create(&std::path::Path::new("/tmp/lang.db")).unwrap();
    base.insert_word("word", "value").unwrap();
    let mut word = base.insert_word("wörd", "translate").unwrap();
    let tags = vec!["noun".to_owned(), "neuter".to_owned()];
    for tag in &tags {
      word.insert_tag(&tag).unwrap();
    }
    let mut forms = HashMap::new();
    forms.insert("AccSg".to_string(), "wordem".to_string());
    forms.insert("VocSg".to_string(), "wordi".to_string());
    for form in &forms {
      word.insert_form(form.0.as_str(), form.1.as_str()).unwrap();
    }
    let mut properties = HashMap::new();
    properties.insert("etymology".to_string(), "word + \"".to_string());
    properties.insert("gender".to_string(), "neuter".to_string());
    for property in &properties {
      word.insert_property(property.0.as_str(), property.1.as_str()).unwrap();
    }

    assert_eq!(word.value().unwrap().as_str(), "translate");
    assert_eq!(word.tags().unwrap(), tags);
    assert_eq!(word.forms().unwrap(), forms);
    assert_eq!(word.properties().unwrap(), properties);
    let words = base.search_word_or_form("wörd").unwrap();
    let words: Vec<_> = words.into_iter().map(|w| w.value().unwrap()).collect();
    assert_eq!(words, vec!["translate".to_string()]);
    let words = base.search_word_or_form("wordem").unwrap();
    let words: Vec<_> = words.into_iter().map(|w| w.value().unwrap()).collect();
    assert_eq!(words, vec!["translate".to_string()]);
    assert_eq!(word.property("etymology").unwrap(), properties["etymology"]);
  }
}

pub struct Bases {
  languages: Arc<RwLock<HashMap<String, Weak<Connection>>>>,
}

impl Bases {
  pub fn new() -> Self {
    Self {
      languages: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  pub fn load_language(&self, lang_id: &str) -> Base {
    unimplemented!()
  }
}

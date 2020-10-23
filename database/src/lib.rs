use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
  pub fn insert_form(&mut self, form: &str, value: &str) -> Result<(), Error> {
    self.base.insert_form(self.word, form, value)
  }
  pub fn insert_property(&mut self, property: &str, value: &str) -> Result<(), Error> {
    self.base.insert_property(self.word, property, value)
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
          word TEXT NOT NULL UNIQUE,
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
    self.connection.write().unwrap().execute(
      "INSERT INTO words (word, value) VALUES (?1, ?2)",
      params![
        word,
        value,
      ]
    )?;
    self.search_word(word)
  }

  fn new_tag(&mut self, tag: &str) -> Result<i64, Error> {
    self.connection.write().unwrap().execute(
      "INSERT INTO tags (tag) VALUES (?1)",
      params![
        tag,
      ]
    )?;
    self.search_tag(tag)
  }

  fn new_form(&mut self, form: &str) -> Result<i64, Error> {
    self.connection.write().unwrap().execute(
      "INSERT INTO forms (form) VALUES (?1)",
      params![
        form,
      ]
    )?;
    self.search_form(form)
  }

  fn new_property(&mut self, property: &str) -> Result<i64, Error> {
    self.connection.write().unwrap().execute(
      "INSERT INTO properties (property) VALUES (?1)",
      params![
        property,
      ]
    )?;
    self.search_property(property)
  }

  pub fn search_word(&self, word: &str) -> Result<Word, Error> {
    let word = {
      let connection = self.connection.read().unwrap();
      let mut stmt = connection.prepare("SELECT id FROM words WHERE word = ?1")?;
      let mut iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
        row.get(0)
      })?;
      iter.next().ok_or_else(|| Error::ValueNotFound(word.to_owned(), "word"))??
    };
    Ok(Word {
      word,
      base: self.clone(),
    })
  }

  pub fn search_word_or_form(&self, word: &str) -> Result<Word, Error> {
    match self.search_word(word) {
      Ok(s) => return Ok(s),
      Err(Error::ValueNotFound(..)) => {},
      Err(e) => return Err(e),
    };
    self.search_form_of_word(word)
  }

  pub fn search_form_of_word(&self, word: &str) -> Result<Word, Error> {
    let word = {
      let connection = self.connection.read().unwrap();
      let mut stmt = connection.prepare(
        "SELECT id FROM words
        INNER JOIN word_forms ON word_forms.word = words.id
        WHERE word_forms.value = ?1")?;
      let mut iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
        row.get(0)
      })?;
      iter.next().ok_or_else(|| Error::ValueNotFound(word.to_owned(), "word form"))??
    };
    Ok(Word {
      word,
      base: self.clone(),
    })
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
    word.insert_tag("noun").unwrap();
    word.insert_tag("neuter").unwrap();
    word.insert_form("AccSg", "wordem").unwrap();
    word.insert_form("VocSg", "wordi").unwrap();
    word.insert_property("etymology", "from word with \"").unwrap();
    word.insert_property("gender", "neuter").unwrap();
    println!("{:?}", word.value().unwrap());
    println!("{:?}", word.tags().unwrap());
    println!("{:?}", word.forms().unwrap());
    println!("{:?}", word.properties().unwrap());
    println!("{:?}", base.search_word_or_form("wörd").unwrap().value().unwrap());
    println!("{:?}", base.search_word_or_form("wordem").unwrap().value().unwrap());
    println!("{:?}", word.property("etymology").unwrap());
  }
}

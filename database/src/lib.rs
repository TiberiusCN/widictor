use rusqlite::{Connection, params};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
  #[error("Value {0} ({1}) not found")]
  ValueNotFound(String, &'static str),
}

pub struct Base {
  connection: Connection,
}

impl Base {
  pub fn create(path: &std::path::Path) -> Result<Self, Error> {
    {
      std::fs::File::create(path)?;
    }
    let me = Self::open(path)?;
    me.connection.execute(
      "CREATE TABLE words (
        id INTEGER PRIMARY KEY,
        word TEXT NOT NULL UNIQUE,
        value TEXT NOT NULL
      )",
      params![]
    )?;
    me.connection.execute(
      "CREATE TABLE tags (
        id INTEGER PRIMARY KEY,
        tag TEXT NOT NULL UNIQUE
      )",
      params![]
    )?;
    me.connection.execute(
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
    // properties
    Ok(me)
  }

  pub fn open(path: &std::path::Path) -> Result<Self, Error> {
    let connection = Connection::open(path)?;
    Ok(Self {
      connection
    })
  }

  pub fn insert_word(&mut self, word: &str, value: &str) -> Result<i64, Error> {
    self.connection.execute(
      "INSERT INTO words (word, value) VALUES (?1, ?2)",
      params![
        word,
        value,
      ]
    )?;
    self.search_word(word)
  }

  fn new_tag(&mut self, tag: &str) -> Result<i64, Error> {
    self.connection.execute(
      "INSERT INTO tags (tag) VALUES (?1)",
      params![
        tag,
      ]
    )?;
    self.search_tag(tag)
  }

  pub fn search_word(&self, word: &str) -> Result<i64, Error> {
    let mut stmt = self.connection.prepare("SELECT id FROM words WHERE word = ?1")?;
    let mut iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(word.to_owned(), "word"))??)
  }

  fn search_tag(&self, tag: &str) -> Result<i64, Error> {
    let mut stmt = self.connection.prepare("SELECT id FROM tags WHERE tag = ?1")?;
    let mut iter = stmt.query_map(params![tag], |row| -> Result<i64, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.next().ok_or_else(|| Error::ValueNotFound(tag.to_owned(), "tag"))??)
  }

  pub fn insert_tag(&mut self, word: i64, tag: &str) -> Result<(), Error> {
    let tag = match self.search_tag(tag) {
      Ok(id) => id,
      Err(Error::ValueNotFound(..)) => self.new_tag(tag)?,
      Err(e) => return Err(e),
    };
    self.connection.execute(
      "INSERT INTO word_tags (word, tag) VALUES (?1, ?2)",
      params![
        word,
        tag,
      ]
    )?;
    Ok(())
  }

  pub fn word_tags(&mut self, word: i64) -> Result<Vec<String>, Error> {
    let mut stmt = self.connection.prepare(
      "SELECT tags.tag FROM tags
        INNER JOIN word_tags ON word_tags.tag = tags.id
        WHERE word_tags.word = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<String, rusqlite::Error> {
      row.get(0)
    })?;
    Ok(iter.collect::<Result<Vec<_>, _>>()?)
  }

  pub fn word_value(&self, word: i64) -> Result<String, Error> {
    let mut stmt = self.connection.prepare("SELECT value FROM words WHERE id = ?1")?;
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
    let word = base.insert_word("wörd", "translate").unwrap();
    base.insert_tag(word, "noun").unwrap();
    base.insert_tag(word, "neuter").unwrap();
    println!("{:?}", base.word_value(word).unwrap());
    println!("{:?}", base.word_tags(word).unwrap());
  }
}

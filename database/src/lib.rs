use rusqlite::{Connection, params};

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  Sqlite(#[from] rusqlite::Error),
  #[error("Word {0} not found")]
  WordNotFound(String),
}

pub struct Base {
  connection: Connection,
}

impl Base {
  pub fn open(path: &std::path::Path) -> Result<Self, Error> {
    let connection = Connection::open(path)?;
    Ok(Self {
      connection
    })
  }

  pub fn insert_word(&mut self, word: &str, value: &str) -> Result<(), Error> {
    self.connection.execute(
      "INSERT INTO words (name, data) VALUES (?1, ?2)",
      params![
        word,
        value,
      ]
    )?;
    Ok(())
  }

  pub fn search_word(&self, word: &str) -> Result<u64, Error> {
    let mut stmt = self.connection.prepare("SELECT id FROM words WHERE name = ?1")?;
    let iter = stmt.query_map(params![word], |row| -> Result<u64, rusqlite::Error> {
      row.get(0)
    })?;
    iter.next().ok_or_else(|| Error::WordNotFound(word.to_owned())).and_then(|v| v?)
  }
}
use std::fs::File;

fn hash(source: &str) -> u64 {
  let mut summ = 0;
  for c in source.chars() {
    summ += c as u64;
  }
  summ
}

pub struct MainFile {
  hash: u32,
  word: u64,
  value: u64,
  w_length: u16,
  v_length: u16,
}

pub struct MainFileMap(File);
pub struct MainFileMapPointer {
  begin: u64,
  count: u32,
}

impl MainFileMapPointer {
  pub fn count(&self) -> u32 {
    self.count
  }
}

impl MainFileMap {
  fn get_words_by_hash(&mut self, hash: u32) -> MainFileMapPointer {
  }
}


pub struct PropertyHeadFile {

}

pub struct Base {
  main_file: File,
  value_file: File,
}

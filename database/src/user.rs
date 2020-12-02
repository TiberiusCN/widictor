use super::*;

#[derive(Clone)]
pub struct Language {
  connection: Arc<RwLock<Connection>>,
}

impl Language {
  fn create(&mut self) -> Result<(), Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "CREATE TABLE words (
        id INTEGER PRIMARY KEY,
        last INTEGER
      )",
      params![]
    )?;

    Ok(())
  }

  pub fn new_word(&mut self, word: i64) -> Result<(), Error> {
    let connection = self.connection.write().unwrap();
    connection.execute(
      "INSERT INTO words (id, last) VALUES (?1, ?2)",
      params![
        word,
        0,
      ]
    )?;
    Ok(())
  }

  pub fn search_or_insert_word(&mut self, word: i64) -> Result<i64, Error> {
    {
      let connection = self.connection.read().unwrap();
      let mut stmt = connection.prepare("SELECT last FROM words WHERE id = ?1")?;
      let mut iter = stmt.query_map(params![word], |row| -> Result<i64, rusqlite::Error> {
        row.get(0)
      })?;
      if let Some(word) = iter.next() {
        return Ok(word?);
      }
    }

    self.new_word(word).map(|_| 0)
  }

  pub fn update_word(&mut self, word: i64, last: i64) -> Result<(), Error> {
    let connection = self.connection.read().unwrap();
    connection.execute("UPDATE words SET last = ?2 WHERE id = ?1", params![word, last])?;
    Ok(())
  }
}

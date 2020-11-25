use std::fs::create_dir;
use std::path::PathBuf;

struct Connection {}
struct Word {}

pub struct User {
  name: String,
  texts: Vec<Text>,
  lesson: Option<Lesson>,
  auto_translate: Option<String>,
  language: String,
}

impl User {
  fn build_text(&mut self, title: &str, text: &str, language: &str) {
    todo!()
  }

  fn open_lesson(&mut self, title: &str) {
    todo!()
  }

  fn extract_next_word(&mut self) -> Word {
    todo!()
  }

  fn success(&mut self) {
    todo!()
  }
}

struct Text {
  title: String,
  language: String,
}

struct Lesson {
  text: String,
  database: Connection,
}

pub struct Config {
  pub user_dir: PathBuf,
  pub language_dir: PathBuf,
}

impl Config {
  fn open_user(&self, user: &str) -> Result<(), std::io::Error> {
    todo!()
  }
  
  fn create_user(&self, user: &str) -> Result<(), std::io::Error> {
    let user_root = self.user_dir.join(user);
    create_dir(user_root)?;
    todo!()
  }
  
  pub fn user(&self, user: &str) -> User {
    todo!()
  }
}

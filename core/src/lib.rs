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

use template::*;
use serde_json::to_writer;

fn main() {
  let text = std::env::var("ENV_MAINWORD").unwrap();
  let lang = std::env::var("ENV_1").unwrap();

  let data = TemplateText {
    conjugation: None,
    subwords: Vec::new(),
    text,
  };
  to_writer(std::io::stdout(), &data).unwrap();
}
